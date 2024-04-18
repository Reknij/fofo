pub mod model;

use std::{borrow::Cow, collections::HashMap};

use anyhow::{bail, Result};
use chrono::Utc;
use futures::TryStreamExt;
use moka::future::Cache;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};
use tracing::error;

use self::model::{
    PostAlgorithmOrder, PostBaseInfo, PostFilterTime, PostInfo, PostStatus, PostToCreate,
    PostToUpdate,
};
use channel_cache::ChannelCacheTask;
use fofo_utils::usizedb;
use shared_core::SharedCore;
use storage::S3Ref;

#[derive(Debug, Clone)]
pub struct PostSystem {
    core: SharedCore,
    s3: S3Ref,
    cached_posts: Cache<usizedb, PostInfo>,
    cached_posts_array: Cache<PostArrayKey, Vec<PostInfo>>,
    cached_posts_count: Cache<PostArrayKey, usizedb>,
    create_task: Option<ChannelCacheTask<(usizedb, PostToCreate), PostInfo>>,
    views_task: Option<ChannelCacheTask<usizedb, ()>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PostArrayKey {
    query_str: String,
    filter_time: PostFilterTime,
}

impl PostSystem {
    pub async fn new(core: SharedCore, s3: S3Ref) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS posts(
                id INTEGER PRIMARY KEY,
                created_by_id INT NOT NULL,
                title VARCHAR(128) NOT NULL,
                content TEXT NOT NULL,
                content_type INT NOT NULL,
                likes INT NOT NULL,
                dislikes INT NOT NULL,
                views INT NOT NULL,
                category_id INT NOT NULL,
                tags TEXT NOT NULL,
                created_at INT NOT NULL,
                last_edit_at INT NOT NULL,
                last_edit_by_id INT NOT NULL,
                status INT NOT NULL,
                total_comment INT NOT NULL,
                total_comment_post INT NOT NULL,
                last_comment_at INT NOT NULL,
                last_comment_by_id INT NOT NULL,
                cover_url TEXT NULL,
                top_index INT NOT NULL DEFAULT 0,

                FOREIGN KEY(created_by_id) REFERENCES users(id),
                FOREIGN KEY(category_id) REFERENCES categories(id),
                FOREIGN KEY(last_edit_by_id) REFERENCES users(id)
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS posts_created_at_index
            on posts (created_at);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        tx.commit().await.unwrap();
        let config = core.get_config();
        let mut this = PostSystem {
            core,
            s3,
            cached_posts: fofo_utils::get_cache_instance(config.clone()).await,
            cached_posts_array: fofo_utils::get_cache_instance(config.clone()).await,
            cached_posts_count: fofo_utils::get_cache_instance(config.clone()).await,
            create_task: None,
            views_task: None,
        };
        let ps = this.clone();
        let _ps2 = this.clone();
        let ps3 = this.clone();
        let create_task = ChannelCacheTask::new(
            "create_posts".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |ptcs| {
                let ps = ps.clone();
                async move {
                    let mut tx = ps.core.begin_unwrap(true).await;
                    let r = ps.create_posts(tx.as_mut(), ptcs).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        );

        let views_task = ChannelCacheTask::new(
            "increment_posts_views".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |views| {
                let ps = ps3.clone();
                let mut map = HashMap::new();
                let len = views.len();
                for i in views {
                    let count = map.entry(i).or_insert(0);
                    *count += 1;
                }

                async move {
                    let mut r = Vec::with_capacity(len);
                    r.resize(len, ());
                    for (post_id, views) in map {
                        let ps = ps.clone();
                        tokio::spawn(async move {
                            let mut tx = ps.core.begin_unwrap(true).await;
                            let r = ps
                                .increment_views_with_count(tx.as_mut(), post_id, views)
                                .await
                                .unwrap();
                            tx.commit_unwrap().await;
                            r
                        });
                    }
                    r
                }
            },
        );
        this.create_task = Some(create_task);
        this.views_task = Some(views_task);
        this
    }

    pub async fn is_exists(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM posts WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some())
    }

    pub async fn increment_views(&self, post_id: usizedb) -> Result<()> {
        match self.views_task.as_ref() {
            Some(task) => {
                let r = task.send(post_id).await?;
                Ok(r)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn increment_views_with_count(
        &self,
        tx: &mut SqliteConnection,
        post_id: usizedb,
        count: usizedb,
    ) -> Result<()> {
        let r = sqlx::query(
            "UPDATE posts SET 
            views = views + ? WHERE id = ?",
        )
        .bind(count)
        .bind(post_id)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() == 0 {
            error!("Can't update views in table posts");
        }

        Ok(())
    }

    pub async fn create_post(&self, user_id: usizedb, mut post: PostToCreate) -> Result<PostInfo> {
        if let Some(url) = &post.cover_url {
            if let Some(key) = self.s3.try_parse_url_to_key(url) {
                if url != key {
                    post.cover_url = Some(key.to_owned())
                }
            }
        }
        match self.create_task.as_ref() {
            Some(task) => {
                let post = task.send((user_id, post)).await?;
                Ok(post)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn create_posts(
        &self,
        tx: &mut SqliteConnection,
        posts: Vec<(usizedb, PostToCreate)>,
    ) -> Result<Vec<PostInfo>> {
        let len = posts.len() as usizedb;
        let now = Utc::now().timestamp() as usizedb;
        let mut query_builder = QueryBuilder::new("INSERT INTO posts (created_by_id, title, content, content_type, likes, dislikes, views, category_id, tags, created_at, last_edit_at, last_edit_by_id, status, last_comment_at, last_comment_by_id, total_comment, total_comment_post, cover_url, top_index) ");
        query_builder.push_values(&posts, |mut b, (user_id, post)| {
            b.push_bind(user_id)
                .push_bind(&post.title)
                .push_bind(&post.content)
                .push_bind(&post.content_type)
                .push_bind(0)
                .push_bind(0)
                .push_bind(0)
                .push_bind(post.category_id)
                .push_bind(fofo_utils::array_to_string(&post.tags))
                .push_bind(now)
                .push_bind(now)
                .push_bind(user_id)
                .push_bind(PostStatus::Active)
                .push_bind(0)
                .push_bind(0)
                .push_bind(0)
                .push_bind(0)
                .push_bind(&post.cover_url)
                .push_bind(post.top_index);
        });
        let query = query_builder.build();
        let r = query.execute(&mut *tx).await?;

        if r.rows_affected() == (len as u64) {
            let mut category_id_map = HashMap::with_capacity(posts.len()); // key is category id, value is total post for this create.
            let mut user_id_map = HashMap::with_capacity(posts.len()); // key is user id, value is total post for this create.
            let base_id = r.last_insert_rowid() as usizedb - len + 1;
            let p: Vec<_> = posts
                .into_iter()
                .enumerate()
                .map(|(i, (user_id, ptc))| {
                    *category_id_map.entry(ptc.category_id).or_insert(0) += 1;
                    *user_id_map.entry(user_id).or_insert(0) += 1;
                    PostInfo {
                        id: base_id + i as usizedb,
                        created_by_id: user_id,
                        title: ptc.title,
                        content: Some(ptc.content),
                        content_type: ptc.content_type,
                        likes: 0,
                        dislikes: 0,
                        views: 0,
                        category_id: ptc.category_id,
                        tags: ptc.tags,
                        created_at: now,
                        last_edit_at: now,
                        last_comment_at: 0,
                        total_comment_post: 0,
                        total_comment: 0,
                        last_edit_by_id: user_id,
                        last_comment_by_id: user_id,
                        status: PostStatus::Active,
                        cover_url: ptc.cover_url,
                        top_index: ptc.top_index,
                    }
                })
                .collect();
            for (category_id, total_post) in category_id_map {
                fofo_utils::increment_category_total_post(&mut *tx, category_id, total_post)
                    .await?;
            }
            for (user_id, total_post) in user_id_map {
                fofo_utils::increment_user_total_post(&mut *tx, user_id, total_post).await?;
            }
            Ok(p)
        } else {
            bail!("Insert failed.")
        }
    }

    pub async fn update_post(
        &self,
        tx: &mut SqliteConnection,
        post_id: usizedb,
        user_id: usizedb,
        mut post: PostToUpdate,
    ) -> Result<Option<PostInfo>> {
        let now = Utc::now().timestamp();
        if let Some(url) = &post.cover_url {
            if let Some(key) = self.s3.try_parse_url_to_key(url) {
                if url != key {
                    post.cover_url = Some(key.to_owned())
                }
            }
        }
        let r = sqlx::query(
            "UPDATE posts SET 
            title = ?,
            content = ?,
            content_type = ?,
            tags = ?,
            last_edit_at = ?,
            last_edit_by_id = ?,
            cover_url = ?,
            top_index=? 
            WHERE id = ?",
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.content_type)
        .bind(fofo_utils::array_to_string(&post.tags))
        .bind(&now)
        .bind(user_id)
        .bind(&post.cover_url)
        .bind(post.top_index)
        .bind(post_id)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() > 1 {
            error!("rows affected is more than 1!");
        }

        Ok(if r.rows_affected() == 1 {
            self.invalidate_cache(post_id).await;
            let post = self.get_post(tx, post_id, true).await?;
            post
        } else {
            None
        })
    }

    async fn invalidate_cache(&self, id: usizedb) {
        self.cached_posts.invalidate(&id).await;
        self.cached_posts_array
            .invalidate_entries_if(move |_, value| {
                for v in value {
                    if v.id == id {
                        return true;
                    }
                }
                false
            })
            .unwrap();
    }

    const POST_NO_CONTENT_COLS: &'static str = "id, title, tags, created_by_id, content_type, likes, dislikes, views, category_id, created_at, last_edit_at, last_edit_by_id, last_comment_at, last_comment_by_id, total_comment, total_comment_post, cover_url, status, top_index";

    pub async fn get_post(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        fetch_content: bool,
    ) -> Result<Option<PostInfo>> {
        Ok(if let Some(cached) = self.cached_posts.get(&id) {
            Some(cached)
        } else {
            let mut rows = sqlx::query("SELECT * FROM posts WHERE id=?")
                .bind(id)
                .fetch(&mut *tx);
            if let Some(row) = rows.try_next().await? {
                let post = self.from_row(row, fetch_content).await;
                self.cached_posts.insert(id, post.clone()).await;
                Some(post)
            } else {
                None
            }
        })
    }

    pub async fn get_post_base(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<PostBaseInfo> {
        Ok(sqlx::query_as("SELECT id, created_by_id, content_type, likes, dislikes, views, category_id, created_at, last_edit_at, last_edit_by_id, last_comment_at, last_comment_by_id, total_comment, total_comment_post, cover_url, status, top_index FROM posts WHERE id=? LIMIT 1")
        .bind(id)
        .fetch_one(&mut *tx).await?)
    }

    async fn from_row(&self, row: SqliteRow, fetch_content: bool) -> PostInfo {
        let status: PostStatus = row.try_get("status").unwrap();
        let cover_url = {
            let cover_url: Option<String> = row.get("cover_url");
            cover_url.map(|url| self.s3.get_real_url(url))
        };
        PostInfo {
            id: row.try_get("id").unwrap(),
            created_by_id: row.try_get("created_by_id").unwrap(),
            title: row.try_get("title").unwrap(),
            content: if status != PostStatus::Banned && fetch_content {
                row.try_get("content").unwrap()
            } else {
                None
            },
            status,
            content_type: row.try_get("content_type").unwrap(),
            likes: row.try_get("likes").unwrap(),
            dislikes: row.try_get("dislikes").unwrap(),
            views: row.try_get("views").unwrap(),
            category_id: row.try_get("category_id").unwrap(),
            tags: fofo_utils::string_to_array(row.try_get("tags").unwrap()).unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            last_edit_at: row.try_get("last_edit_at").unwrap(),
            last_edit_by_id: row.try_get("last_edit_by_id").unwrap(),
            last_comment_at: row.try_get("last_comment_at").unwrap(),
            last_comment_by_id: row.try_get("last_comment_by_id").unwrap(),
            total_comment: row.try_get("total_comment").unwrap(),
            total_comment_post: row.try_get("total_comment_post").unwrap(),
            top_index: row.try_get("top_index").unwrap(),
            cover_url,
        }
    }

    fn get_order_sql(&self, order: PostAlgorithmOrder) -> &'static str {
        match order {
            PostAlgorithmOrder::Hot => {
                "((likes + dislikes) * 500) + views + (total_comment * 100) DESC"
            }
            PostAlgorithmOrder::Views => "views DESC",
            PostAlgorithmOrder::Likes => "likes DESC",
            PostAlgorithmOrder::Newest => "created_at DESC",
        }
    }

    pub async fn get_postlinks_with_algorithm(
        &self,
        tx: &mut SqliteConnection,
        index: usizedb,
        limit: usizedb,
        order: PostAlgorithmOrder,
        filter_time: PostFilterTime,
        category_id: Option<usizedb>,
        created_by_id: Option<usizedb>,
        distinct: bool,
        top_order_enable: bool,
    ) -> Result<Vec<PostInfo>> {
        let order_by = self.get_order_sql(order);
        let offset = index * limit;
        let mut conds = Vec::with_capacity(3);
        if let Some(category_id) = category_id {
            conds.push(format!("category_id={category_id}"))
        }
        if let Some(created_by_id) = created_by_id {
            conds.push(format!("created_by_id={created_by_id}"))
        }
        if !filter_time.is_lifetime() {
            let time = filter_time.to_timestamp(true);
            conds.push(if top_order_enable {
                format!("(top_index > 0 OR created_at >= {time})")
            } else {
                format!("created_at >= {time}")
            });
        }
        let where_conds = if conds.is_empty() {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!("WHERE {}", conds.join(" AND ")))
        };
        let top_order = if top_order_enable {
            "top_index DESC,"
        } else {
            ""
        };

        let cols: &'static str = Self::POST_NO_CONTENT_COLS;
        let query_str = if distinct {
            format!(
                "WITH cte AS (
                    SELECT {cols}, ROW_NUMBER() OVER (PARTITION BY created_by_id ORDER BY {top_order} {order_by}) AS rn FROM posts {where_conds} ORDER BY {top_order} {order_by}
                  )
                  SELECT {cols} FROM cte WHERE rn = 1 LIMIT {limit} OFFSET {offset};",
            )
        } else {
            format!(
                "SELECT {cols} FROM posts {where_conds} ORDER BY {top_order} {order_by} LIMIT {limit} OFFSET {offset};"
            )
        };
        let key = PostArrayKey {
            query_str,
            filter_time,
        };

        Ok(if let Some(cached) = self.cached_posts_array.get(&key) {
            cached
        } else {
            let mut arr = Vec::with_capacity(limit as _);
            {
                let mut rows = sqlx::query(&key.query_str).fetch(&mut *tx); // bind time and status because `conds.push(format!("created_at >= ? AND status != ?"));`
                while let Some(row) = rows.try_next().await? {
                    arr.push(self.from_row(row, false).await)
                }
            }
            self.cached_posts_array.insert(key, arr.clone()).await;
            arr
        })
    }

    pub async fn get_all_post_count(
        &self,
        tx: &mut SqliteConnection,
        filter_time: PostFilterTime,
        category_id: Option<usizedb>,
        created_by_id: Option<usizedb>,
        distinct: bool,
        top_order_enable: bool,
    ) -> Result<usizedb> {
        // execute a query to get the row count of a table
        let mut conds = Vec::with_capacity(3);
        if let Some(category_id) = category_id {
            conds.push(format!("category_id={category_id}"))
        }
        if let Some(created_by_id) = created_by_id {
            conds.push(format!("created_by_id={created_by_id}"))
        }
        if !filter_time.is_lifetime() {
            let time = filter_time.to_timestamp(true);
            conds.push(if top_order_enable {
                format!("(top_index > 0 OR created_at >= {time})")
            } else {
                format!("created_at >= {time}")
            });
        }
        let where_conds = if conds.is_empty() {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!("WHERE {}", conds.join(" AND ")))
        };
        let query_str = if distinct {
            format!(
                "WITH cte AS (
                    SELECT likes, dislikes, views, total_comment, created_at, created_by_id, category_id, ROW_NUMBER() OVER (PARTITION BY created_by_id) AS rn FROM posts {where_conds}
                  )
                  SELECT COUNT(*) FROM cte WHERE rn = 1;",
            )
        } else {
            format!("SELECT COUNT(*) FROM posts {where_conds};",)
        };
        let key = PostArrayKey {
            query_str,
            filter_time,
        };

        Ok(if let Some(cached) = self.cached_posts_count.get(&key) {
            cached
        } else {
            let count: usizedb = sqlx::query_scalar(&key.query_str)
                .fetch_one(&mut *tx)
                .await?; // bind time and status because `conds.push(format!("created_at >= ? AND status != ?"));`
            self.cached_posts_count.insert(key, count).await;
            count
        })
    }

    pub async fn set_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        status: PostStatus,
    ) -> Result<bool> {
        let r = sqlx::query("UPDATE posts SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        self.invalidate_cache(id).await;
        Ok(r.rows_affected() == 1)
    }

    pub async fn get_status(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<PostStatus> {
        let r = sqlx::query("SELECT status FROM posts WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(r.get("status"))
    }
}
