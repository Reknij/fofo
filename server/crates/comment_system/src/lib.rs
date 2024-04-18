use std::collections::HashMap;

use anyhow::{bail, Result};
use channel_cache::ChannelCacheTask;
use chrono::Utc;
use fofo_utils::usizedb;
use futures::TryStreamExt;
use moka::future::Cache;
use shared_core::SharedCore;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};

use self::model::{
    CommentBaseInfo, CommentInfo, CommentStatus, CommentToCreate, CommentToUpdate, GetCommentsSort,
};

pub mod model;

#[derive(Debug, Clone)]
pub struct CommentSystem {
    cached_comments: Cache<usizedb, CommentInfo>,
    cached_comments_array: Cache<String, Vec<CommentInfo>>,
    core: SharedCore,
    create_task: Option<ChannelCacheTask<(usizedb, CommentToCreate), CommentInfo>>,
}

impl CommentSystem {
    pub async fn new(core: SharedCore) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS comments(
                id INTEGER PRIMARY KEY,
                parent_id INT NOT NULL,
                post_id INT NOT NULL,
                category_id INT NOT NULL,
                reply_comment_id INT NOT NULL,
                reply_user_id INT NOT NULL,
                created_at INT NOT NULL,
                created_by_id INT NOT NULL,
                last_edit_at INT NOT NULL,
                last_edit_by_id INT NOT NULL,
                content TEXT NOT NULL,
                content_type INT NOT NULL,
                likes INT NOT NULL,
                dislikes INT NOT NULL,
                status INT NOT NULL,
                total_comment INT NOT NULL,
                last_comment_by_id INT NOT NULL,
                last_comment_at INT NOT NULL,
                top_index INT NOT NULL DEFAULT 0,

                FOREIGN KEY(post_id) REFERENCES posts(id),
                FOREIGN KEY(created_by_id) REFERENCES users(id),
                FOREIGN KEY(last_edit_by_id) REFERENCES users(id)
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
        let config = core.get_config();
        let mut this = CommentSystem {
            core,
            cached_comments: fofo_utils::get_cache_instance(config.clone()).await,
            cached_comments_array: fofo_utils::get_cache_instance(config.clone()).await,
            create_task: None,
        };
        let cs = this.clone();
        let create_task = ChannelCacheTask::<(usizedb, CommentToCreate), CommentInfo>::new(
            "create_comments".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |cmcs| {
                let cs = cs.clone();
                async move {
                    let mut tx = cs.core.begin_unwrap(true).await;
                    let r = cs.create_comments(tx.as_mut(), cmcs).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        );
        this.create_task = Some(create_task);
        this
    }

    pub async fn is_exists(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM comments WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some())
    }

    pub async fn create_comment(
        &self,
        user_id: usizedb,
        comment: CommentToCreate,
    ) -> Result<CommentInfo> {
        match self.create_task.as_ref() {
            Some(task) => {
                let comment = task.send((user_id, comment)).await?;
                Ok(comment)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn create_comments(
        &self,
        tx: &mut SqliteConnection,
        comments: Vec<(usizedb, CommentToCreate)>,
    ) -> Result<Vec<CommentInfo>> {
        let len = comments.len() as usizedb;
        let now = Utc::now().timestamp() as usizedb;
        let mut query_builder = QueryBuilder::new("INSERT INTO comments (parent_id, created_at, created_by_id, last_edit_at, last_edit_by_id, category_id, post_id, reply_user_id, reply_comment_id, content, content_type, likes, dislikes, status, total_comment, last_comment_at, last_comment_by_id, top_index) ");
        query_builder.push_values(&comments, |mut b, (user_id, ctc)| {
            b.push_bind(ctc.parent_id)
                .push_bind(now)
                .push_bind(user_id)
                .push_bind(now)
                .push_bind(user_id)
                .push_bind(ctc.category_id)
                .push_bind(ctc.post_id)
                .push_bind(ctc.reply_user_id)
                .push_bind(ctc.reply_comment_id)
                .push_bind(&ctc.content)
                .push_bind(ctc.content_type)
                .push_bind(0)
                .push_bind(0)
                .push_bind(CommentStatus::Active)
                .push_bind(0)
                .push_bind(0)
                .push_bind(0)
                .push_bind(ctc.top_index);
        });

        let query = query_builder.build();
        let r = query.execute(&mut *tx).await?;
        if r.rows_affected() == (len as u64) {
            let base_id = r.last_insert_rowid() as usizedb - len + 1;
            let mut reply_posts = HashMap::with_capacity(comments.len());
            let mut reply_comments = HashMap::with_capacity(comments.len());
            let mut user_id_map = HashMap::with_capacity(comments.len()); // key is user id, value is total post for this create.
            let comments: Vec<_> = comments
                .into_iter()
                .enumerate()
                .map(|(i, (user_id, ctc))| {
                    let comment = CommentInfo {
                        id: base_id + i as usizedb,
                        parent_id: ctc.parent_id,
                        created_at: now,
                        last_comment_at: 0,
                        last_comment_by_id: 0,
                        created_by_id: user_id,
                        last_edit_at: now,
                        last_edit_by_id: user_id,
                        post_id: ctc.post_id,
                        category_id: ctc.category_id,
                        reply_user_id: ctc.reply_user_id,
                        reply_comment_id: ctc.reply_comment_id,
                        content: ctc.content,
                        content_type: ctc.content_type,
                        likes: 0,
                        dislikes: 0,
                        total_comment: 0,
                        top_index: ctc.top_index,
                        status: CommentStatus::Active,
                    };
                    *user_id_map.entry(user_id).or_insert(0) += 1;

                    // total_comment is all comment of post. total_comment_post is comment reply post directly.
                    reply_posts
                        .entry(comment.post_id)
                        .and_modify(|(_last_comment_index, total_comment, total_comment_post)| {
                            *total_comment += 1;
                            if comment.reply_comment_id == 0 {
                                // directly reply to post
                                *total_comment_post += 1;
                            }
                        })
                        .or_insert((i, 1, if comment.reply_comment_id == 0 { 1 } else { 0 }));

                    if comment.reply_comment_id > 0 {
                        reply_comments
                            .entry(comment.reply_comment_id)
                            .and_modify(|(_last_comment_index, total_comment)| {
                                *total_comment += 1;
                            })
                            .or_insert((i, 1));
                    }

                    comment // return back
                })
                .collect();

            for (post_id, (i, total_comment, total_comment_post)) in reply_posts {
                fofo_utils::increment_post_total_comment(
                    tx,
                    post_id,
                    comments[i].created_at,
                    comments[i].created_by_id,
                    total_comment,
                    total_comment_post,
                )
                .await?;
            }
            for (comment_id, (i, total_comment)) in reply_comments {
                fofo_utils::increment_comment_total_sub_comments(
                    tx,
                    comment_id,
                    comments[i].created_at,
                    comments[i].created_by_id,
                    total_comment,
                )
                .await?;
            }
            for (user_id, total_comment) in user_id_map {
                fofo_utils::increment_user_total_comment(&mut *tx, user_id, total_comment).await?;
            }
            Ok(comments)
        } else {
            bail!("Insert failed.")
        }
    }

    pub async fn update_comment(
        &self,
        tx: &mut SqliteConnection,
        comment_id: usizedb,
        user_id: usizedb,
        comment: CommentToUpdate,
    ) -> Result<Option<CommentInfo>> {
        let now = Utc::now().timestamp();

        let r = sqlx::query(
            "UPDATE comments SET last_edit_at=?, last_edit_by_id=?, content=?, content_type=?, top_index=? WHERE id=?",
        )
        .bind(now)
        .bind(user_id)
        .bind(&comment.content)
        .bind(&comment.content_type)
        .bind(comment.top_index)
        .bind(comment_id)
        .execute(&mut *tx)
        .await?;

        if r.rows_affected() == 1 {
            let comment = self.get_comment(tx, comment_id).await?;
            self.invalidate_cache(comment_id).await;
            Ok(comment)
        } else {
            bail!("Insert failed.")
        }
    }

    async fn invalidate_cache(&self, id: usizedb) {
        self.cached_comments.invalidate(&id).await;
        self.cached_comments_array
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

    fn from_row(&self, row: SqliteRow) -> CommentInfo {
        let status: CommentStatus = row.try_get("status").unwrap();
        let content = match status {
            CommentStatus::Active => row.try_get("content").unwrap(),
            CommentStatus::Banned => String::new(),
        };
        CommentInfo {
            id: row.try_get("id").unwrap(),
            parent_id: row.try_get("parent_id").unwrap(),
            category_id: row.try_get("category_id").unwrap(),
            post_id: row.try_get("post_id").unwrap(),
            reply_user_id: row.try_get("reply_user_id").unwrap(),
            reply_comment_id: row.try_get("reply_comment_id").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            last_comment_at: row.try_get("last_comment_at").unwrap(),
            created_by_id: row.try_get("created_by_id").unwrap(),
            last_comment_by_id: row.try_get("last_comment_by_id").unwrap(),
            last_edit_at: row.try_get("last_edit_at").unwrap(),
            last_edit_by_id: row.try_get("last_edit_by_id").unwrap(),
            content,
            content_type: row.try_get("content_type").unwrap(),
            likes: row.try_get("likes").unwrap(),
            dislikes: row.try_get("dislikes").unwrap(),
            total_comment: row.try_get("total_comment").unwrap(),
            top_index: row.try_get("top_index").unwrap(),
            status,
        }
    }

    pub async fn get_comment(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<Option<CommentInfo>> {
        Ok(if let Some(cached) = self.cached_comments.get(&id) {
            Some(cached)
        } else {
            let mut rows = sqlx::query("SELECT * FROM comments WHERE id=?")
                .bind(id)
                .fetch(&mut *tx);
            if let Some(row) = rows.try_next().await? {
                let v = self.from_row(row);
                self.cached_comments.insert(id, v.clone()).await;
                Some(v)
            } else {
                None
            }
        })
    }

    pub async fn get_comment_base(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<CommentBaseInfo> {
        Ok(sqlx::query_as("SELECT id, parent_id, created_at, created_by_id, last_edit_at, last_edit_by_id, content_type, post_id, category_id, reply_user_id, reply_comment_id, likes, dislikes, status, total_comment, last_comment_at, last_comment_by_id, top_index FROM comments WHERE id=? LIMIT 1")
        .bind(id)
        .fetch_one(&mut *tx).await?)
    }

    pub async fn get_comments(
        &self,
        tx: &mut SqliteConnection,
        post_id: usizedb,
        parent_id: usizedb,
        sort: GetCommentsSort,
        index: usizedb,
        limit: usizedb,
        desc: bool,
        top_order_enable: bool,
    ) -> Result<Vec<CommentInfo>> {
        let offset = index * limit;
        let order_by = match sort {
            GetCommentsSort::Id => "id",
            GetCommentsSort::Likes => "likes",
            GetCommentsSort::Dislikes => "dislikes",
            GetCommentsSort::TotalPost => "total_post",
        };
        let top_order = if top_order_enable {
            "top_index DESC,"
        } else {
            ""
        };
        let q = if desc {
            format!("SELECT * FROM comments WHERE post_id={post_id} AND parent_id={parent_id} ORDER BY {top_order} {order_by} DESC LIMIT {limit} OFFSET {offset}")
        } else {
            format!("SELECT * FROM comments WHERE post_id={post_id} AND parent_id={parent_id} ORDER BY {top_order} {order_by} ASC LIMIT {limit} OFFSET {offset}")
        };

        Ok(if let Some(cached) = self.cached_comments_array.get(&q) {
            cached
        } else {
            let mut arr = vec![];
            {
                let mut rows = sqlx::query(&q).fetch(&mut *tx);
                while let Some(row) = rows.try_next().await? {
                    arr.push(self.from_row(row))
                }
            }
            self.cached_comments_array
                .insert(q.to_owned(), arr.clone())
                .await;
            arr
        })
    }

    pub async fn set_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        status: CommentStatus,
    ) -> Result<bool> {
        let r = sqlx::query("UPDATE comments SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        self.invalidate_cache(id).await;
        Ok(r.rows_affected() == 1)
    }

    pub async fn get_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<CommentStatus> {
        let r = sqlx::query("SELECT status FROM posts WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(r.get("status"))
    }
}
