pub mod model;

use anyhow::{bail, Result};
use fofo_utils::usizedb;
use futures::TryStreamExt;
use model::{Category, CategoryToCreate, CategoryToUpdate};
use moka::future::Cache;
use shared_core::SharedCore;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};
use storage::S3Ref;
use tracing::warn;
use user_system::model::{UserInfo, UserType};

use self::model::{CategoryLevels, CategoryStatus, GetCategoriesSort};

#[derive(Debug, Clone)]
pub struct CategorySystem {
    core: SharedCore,
    s3: S3Ref,
    cached_categories: Cache<usizedb, Category>,
    cached_categories_array: Cache<String, Vec<Category>>,
}

impl CategorySystem {
    pub async fn new(core: SharedCore, s3: S3Ref) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS categories(
                id INTEGER PRIMARY KEY,
                title VARCHAR(128) NOT NULL,
                description TEXT NOT NULL,
                description_content_type INT NOT NULL,
                status INT NOT NULL,
                read_level INT NOT NULL,
                write_level INT NOT NULL,
                comment_level INT NOT NULL,
                total_post INT NOT NULL,
                cover_url TEXT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS category_groups(
                category_id INT NOT NULL,
                group_id INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS category_moderators(
                category_id INT NOT NULL,
                user_id INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS category_title_index
            on categories (title);
            CREATE INDEX IF NOT EXISTS category_groups_category
            on category_groups (category_id);
            CREATE INDEX IF NOT EXISTS category_groups_group
            on category_groups (group_id);
            CREATE INDEX IF NOT EXISTS category_moderators_category
            on category_moderators (category_id);
            CREATE INDEX IF NOT EXISTS category_moderators_user
            on category_moderators (user_id);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        tx.commit().await.unwrap();
        let config = core.get_config().clone();
        let this = CategorySystem {
            core,
            s3,
            cached_categories: fofo_utils::get_cache_instance(config.clone()).await,
            cached_categories_array: fofo_utils::get_cache_instance(config).await,
        };
        this
    }

    pub async fn is_exists(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM categories WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some())
    }

    pub async fn is_exists_duplicate_title(
        &self,
        tx: &mut SqliteConnection,
        title: &str,
        id: Option<usizedb>,
    ) -> Result<bool> {
        if let Some(row) = sqlx::query("SELECT id FROM categories WHERE title=? LIMIT 1")
            .bind(title)
            .fetch_optional(&mut *tx)
            .await?
        {
            return Ok(match id {
                Some(id) if row.get::<'_, usizedb, _>("id") == id => false,
                Some(_) | None => true,
            });
        }
        Ok(false)
    }

    pub async fn create_category(&self, mut category: CategoryToCreate) -> Result<Category> {
        if let Some(url) = &category.cover_url {
            if let Some(key) = self.s3.try_parse_url_to_key(url) {
                if url != key {
                    category.cover_url = Some(key.to_owned())
                }
            }
        }
        let mut tx = self.core.begin_unwrap(false).await;
        let r = sqlx::query("INSERT INTO categories (title, description, description_content_type, status, read_level, write_level, comment_level, total_post, cover_url) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&category.title)
            .bind(&category.description)
            .bind(&category.description_content_type)
            .bind(&category.status)
            .bind(&category.read_level)
            .bind(&category.write_level)
            .bind(&category.comment_level)
            .bind(0)
            .bind(&category.cover_url)
            .execute(tx.as_mut())
            .await?;

        if r.rows_affected() == 1 {
            let id = r.last_insert_rowid() as _;
            self.insert_groups_and_moderators(
                tx.as_mut(),
                id,
                &category.group_ids,
                &category.moderator_ids,
            )
            .await?;

            let category = Category {
                id,
                title: category.title,
                description: Some(category.description),
                description_content_type: category.description_content_type,
                status: category.status,
                read_level: category.read_level,
                write_level: category.write_level,
                comment_level: category.comment_level,
                moderator_ids: category.moderator_ids,
                group_ids: category.group_ids,
                total_post: 0,
                cover_url: category.cover_url,
            };
            tx.commit().await?;
            Ok(category)
        } else {
            bail!("Insert failed.")
        }
    }

    pub async fn update_category(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        mut category: CategoryToUpdate,
    ) -> Result<Option<Category>> {
        if let Some(url) = &category.cover_url {
            if let Some(key) = self.s3.try_parse_url_to_key(url) {
                if url != key {
                    category.cover_url = Some(key.to_owned())
                }
            }
        }
        let r = sqlx::query("UPDATE categories SET title=?, description=?, description_content_type=?, status=?, read_level=?, write_level=?, comment_level=?, cover_url=? WHERE id=?")
            .bind(&category.title)
            .bind(&category.description)
            .bind(&category.description_content_type)
            .bind(&category.status)
            .bind(&category.read_level)
            .bind(&category.write_level)
            .bind(&category.comment_level)
            .bind(id)
            .bind(&category.cover_url)
            .execute(&mut *tx)
            .await?;

        if r.rows_affected() == 1 {
            self.insert_groups_and_moderators(
                &mut *tx,
                id,
                &category.group_ids,
                &category.moderator_ids,
            )
            .await?;
            self.invalidate_cache(id).await;
            let total_post = sqlx::query("SELECT total_post FROM categories WHERE id=?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?
                .get("total_post");
            return Ok(Some(Category {
                id,
                title: category.title,
                description: Some(category.description),
                description_content_type: category.description_content_type,
                status: category.status,
                read_level: category.read_level,
                write_level: category.write_level,
                comment_level: category.comment_level,
                moderator_ids: category.moderator_ids,
                group_ids: category.group_ids,
                total_post,
                cover_url: category.cover_url,
            }));
        }

        Ok(None)
    }

    async fn insert_groups_and_moderators(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        groups: &Vec<usizedb>,
        moderators: &Vec<usizedb>,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM category_groups WHERE category_id=?;
        DELETE FROM category_moderators WHERE category_id=?;",
        )
        .bind(category_id)
        .execute(&mut *tx)
        .await?;
        if groups.len() > 0 {
            QueryBuilder::new("INSERT INTO category_groups (category_id, group_id) ")
                .push_values(groups, |mut b, id| {
                    b.push_bind(category_id).push_bind(id);
                })
                .build()
                .execute(&mut *tx)
                .await?;
        }
        if moderators.len() > 0 {
            QueryBuilder::new("INSERT INTO category_moderators (category_id, user_id) ")
                .push_values(moderators, |mut b, id| {
                    b.push_bind(category_id).push_bind(id);
                })
                .build()
                .execute(&mut *tx)
                .await?;
        }
        Ok(())
    }

    async fn invalidate_cache(&self, id: usizedb) {
        self.cached_categories.invalidate(&id).await;
        self.cached_categories_array
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

    async fn from_row(
        &self,
        tx: &mut SqliteConnection,
        row: SqliteRow,
        include_content: bool,
    ) -> Category {
        let cover_url = {
            let cover_url: Option<String> = row.get("cover_url");
            cover_url.map(|url| self.s3.get_real_url(url))
        };
        let id = row.try_get("id").unwrap();
        Category {
            id,
            title: row.try_get("title").unwrap(),
            description: if include_content {
                Some(row.try_get("description").unwrap())
            } else {
                None
            },
            description_content_type: row.try_get("description_content_type").unwrap(),
            status: row.try_get("status").unwrap(),
            read_level: row.try_get("read_level").unwrap(),
            write_level: row.try_get("write_level").unwrap(),
            comment_level: row.try_get("comment_level").unwrap(),
            group_ids: sqlx::query("SELECT group_id FROM category_groups WHERE category_id=?")
                .bind(id)
                .fetch_all(&mut *tx)
                .await
                .map(|rows| rows.into_iter().map(|row| row.get(0)).collect())
                .unwrap_or(Vec::new()),
            moderator_ids: sqlx::query(
                "SELECT user_id FROM category_moderators WHERE category_id=?",
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await
            .map(|rows| rows.into_iter().map(|row| row.get(0)).collect())
            .unwrap_or(Vec::new()),
            total_post: row.try_get("total_post").unwrap(),
            cover_url,
        }
    }

    pub async fn get_category(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<Option<Category>> {
        Ok(if let Some(cached) = self.cached_categories.get(&id) {
            Some(cached)
        } else {
            let r = sqlx::query("SELECT * FROM categories WHERE id = ?")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?;
            if let Some(row) = r {
                let v = self.from_row(tx, row, true).await;
                self.cached_categories.insert(id, v.clone()).await;
                Some(v)
            } else {
                None
            }
        })
    }

    pub async fn get_categories(
        &self,
        tx: &mut SqliteConnection,
        index: usizedb,
        limit: usizedb,
        sort: GetCategoriesSort,
        desc: bool,
    ) -> Result<Vec<Category>> {
        let offset = index * limit;
        let order_by = match sort {
            GetCategoriesSort::Id => "id",
            GetCategoriesSort::Title => "title",
            GetCategoriesSort::TotalPost => "total_post",
        };
        let query_str = if desc {
            format!(
                "SELECT * FROM categories ORDER BY {order_by} DESC LIMIT {limit} OFFSET {offset}"
            )
        } else {
            format!(
                "SELECT * FROM categories ORDER BY {order_by} ASC LIMIT {limit} OFFSET {offset}"
            )
        };

        Ok(
            if let Some(cached) = self.cached_categories_array.get(&query_str) {
                cached
            } else {
                let mut categories = Vec::with_capacity(limit as _);
                {
                    let r = sqlx::query(&query_str).fetch_all(&mut *tx).await?;
                    for row in r {
                        categories.push(self.from_row(tx, row, false).await);
                    }
                }
                self.cached_categories_array
                    .insert(query_str, categories.clone())
                    .await;
                categories
            },
        )
    }

    pub async fn get_count(
        &self,
        tx: &mut SqliteConnection,
        _index: usizedb,
        _limit: usizedb,
    ) -> Result<usizedb> {
        Ok(sqlx::query("SELECT COUNT(*) FROM categories")
            .fetch_optional(&mut *tx)
            .await?
            .map(|row| row.get(0))
            .unwrap_or(0))
    }

    pub async fn delete_category(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        let r = sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if r.rows_affected() > 1 {
            warn!("rows affected is more than 1.")
        }
        Ok(r.rows_affected() == 1)
    }

    pub async fn set_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        status: CategoryStatus,
    ) -> Result<bool> {
        let r = sqlx::query("UPDATE categories SET status = ? WHERE id = ?")
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
    ) -> Result<CategoryStatus> {
        let r = sqlx::query("SELECT status FROM categories WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(r.get("status"))
    }

    pub async fn is_user_inside_group(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        user: &UserInfo,
    ) -> Result<bool> {
        for group_id in &user.group_ids {
            if sqlx::query(
                "SELECT 1 FROM category_groups WHERE category_id=? AND group_id=? LIMIT 1",
            )
            .bind(category_id)
            .bind(group_id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn get_category_levels(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
    ) -> Result<CategoryLevels> {
        Ok(sqlx::query_as(
            "SELECT read_level, write_level, comment_level FROM categories WHERE id=? LIMIT 1",
        )
        .bind(category_id)
        .fetch_one(&mut *tx)
        .await?)
    }

    pub async fn can_manage(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        user: Option<&UserInfo>,
    ) -> Result<bool> {
        Ok(match user {
            Some(user) if user.is_admin() || sqlx::query("SELECT 1 FROM category_moderators WHERE category_id=? AND user_id=? LIMIT 1")
            .bind(category_id)
            .bind(user.id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some() => true,
            Some(_) | None=> false,
        })
    }

    pub async fn can_write(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        user: Option<&UserInfo>,
    ) -> Result<bool> {
        Ok(match user {
            Some(user)
                if self.get_category_levels(tx, category_id).await?.write_level
                    <= user.user_type
                    && self.is_user_inside_group(tx, category_id, user).await? =>
            {
                true
            }
            Some(_) | None => self.can_manage(tx, category_id, user).await?,
        })
    }

    pub async fn can_comment(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        user: Option<&UserInfo>,
    ) -> Result<bool> {
        Ok(match user {
            Some(user)
                if self
                    .get_category_levels(tx, category_id)
                    .await?
                    .comment_level
                    <= user.user_type
                    && self.is_user_inside_group(tx, category_id, user).await? =>
            {
                true
            }
            Some(_) | None => self.can_manage(tx, category_id, user).await?,
        })
    }

    pub async fn can_read(
        &self,
        tx: &mut SqliteConnection,
        category_id: usizedb,
        user: Option<&UserInfo>,
    ) -> Result<bool> {
        let levels = self.get_category_levels(tx, category_id).await?;
        Ok(match user {
            Some(user)
                if levels.read_level <= user.user_type
                    && self.is_user_inside_group(tx, category_id, user).await? =>
            {
                true
            }
            None if levels.read_level == UserType::Guest => true,
            Some(_) | None => self.can_manage(tx, category_id, user).await?,
        })
    }
}
