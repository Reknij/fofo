use fofo_utils::{usizedb, ContentType};
use futures::TryStreamExt;
use moka::future::Cache;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

use tracing::warn;

use shared_core::SharedCore;

use self::model::{GetGroupsSort, Group, GroupStatus, GroupToCreateUpdate};
use anyhow::{bail, Result};

pub mod model;

#[derive(Debug, Clone)]
pub struct GroupSystem {
    core: SharedCore,
    cached_groups: Cache<usizedb, Group>,
    cached_groups_array: Cache<String, Vec<Group>>,
}

impl GroupSystem {
    pub async fn new(core: SharedCore) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS groups(
                id INTEGER PRIMARY KEY,
                title varchar(128) NOT NULL,
                description TEXT NOT NULL,
                description_content_type INT NOT NULL,
                status INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS group_title_index
            on groups (title);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        sqlx::query(
            "INSERT OR IGNORE INTO groups (id, title, description, description_content_type, status)
            VALUES (1,?,?,?,?);",
        )
        .bind("General")
        .bind("# General group\nThe group is for all new users.")
        .bind(ContentType::Markdown)
        .bind(GroupStatus::Active)
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        tx.commit().await.unwrap();
        let config = core.get_config();
        let this = GroupSystem {
            core,
            cached_groups: fofo_utils::get_cache_instance(config.clone()).await,
            cached_groups_array: fofo_utils::get_cache_instance(config).await,
        };
        this
    }

    pub async fn is_exists(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM groups WHERE id=? LIMIT 1")
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
        if let Some(row) = sqlx::query("SELECT id FROM groups WHERE title=? LIMIT 1")
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

    pub async fn create_group(&self, group: GroupToCreateUpdate) -> Result<Group> {
        let mut tx = self.core.begin().await?;
        let r = sqlx::query("INSERT INTO groups (title, description, description_content_type, status) VALUES (?, ?, ?, ?)")
            .bind(&group.title)
            .bind(&group.description)
            .bind(&group.description_content_type)
            .bind(&group.status)
            .execute(tx.as_mut())
            .await?;

        if r.rows_affected() == 1 {
            tx.commit().await?;
            let group = Group {
                id: r.last_insert_rowid() as _,
                title: group.title,
                description: group.description,
                description_content_type: group.description_content_type,
                status: group.status,
            };
            Ok(group)
        } else {
            bail!("Insert failed.")
        }
    }

    pub async fn update_group(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        group: GroupToCreateUpdate,
    ) -> Result<Option<Group>> {
        let r = sqlx::query("UPDATE groups SET title=?, description=?, description_content_type=?, status=? WHERE id=?")
            .bind(&group.title)
            .bind(&group.description)
            .bind(&group.description_content_type)
            .bind(&group.status)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if r.rows_affected() == 1 {
            self.invalidate_cache(id).await;
            return Ok(Some(Group {
                id,
                title: group.title,
                description: group.description,
                description_content_type: group.description_content_type,
                status: group.status,
            }));
        }
        Ok(None)
    }

    async fn invalidate_cache(&self, id: usizedb) {
        self.cached_groups.invalidate(&id).await;
        self.cached_groups_array
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

    pub async fn delete_group(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        let r = sqlx::query("DELETE FROM groups WHERE id = ? LIMIT 1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if r.rows_affected() > 1 {
            warn!("rows affected is more than 1.")
        }
        Ok(r.rows_affected() == 1)
    }

    fn from_row(&self, row: SqliteRow) -> Group {
        Group {
            id: row.try_get("id").unwrap(),
            title: row.try_get("title").unwrap(),
            description: row.try_get("description").unwrap(),
            description_content_type: row.try_get("description_content_type").unwrap(),
            status: row.try_get("status").unwrap(),
        }
    }

    pub async fn get_group(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<Option<Group>> {
        Ok(if let Some(cached) = self.cached_groups.get(&id) {
            Some(cached)
        } else {
            let mut r = sqlx::query("SELECT * FROM groups WHERE id = ?")
                .bind(id)
                .fetch(&mut *tx);
            if let Some(row) = r.try_next().await? {
                let v = self.from_row(row);
                self.cached_groups.insert(id, v.clone()).await;
                Some(v)
            } else {
                None
            }
        })
    }

    pub async fn get_count(
        &self,
        tx: &mut SqliteConnection,
        _index: usizedb,
        _limit: usizedb,
    ) -> Result<usizedb> {
        Ok(sqlx::query("SELECT COUNT(*) FROM groups")
            .fetch_optional(&mut *tx)
            .await?
            .map(|row| row.get(0))
            .unwrap_or(0)) // Because counting is too slow. Return directly 100 pages.
    }

    pub async fn get_groups(
        &self,
        tx: &mut SqliteConnection,
        index: usizedb,
        limit: usizedb,
        sort: GetGroupsSort,
        desc: bool,
    ) -> Result<Vec<Group>> {
        let offset = index * limit;
        let order_by = match sort {
            GetGroupsSort::Id => "id",
            GetGroupsSort::Title => "title",
        };
        let query_str = if desc {
            format!("SELECT * FROM groups ORDER BY {order_by} DESC LIMIT {limit} OFFSET {offset}")
        } else {
            format!("SELECT * FROM groups ORDER BY {order_by} ASC LIMIT {limit} OFFSET {offset}")
        };

        Ok(
            if let Some(cached) = self.cached_groups_array.get(&query_str) {
                cached
            } else {
                let mut groups = Vec::with_capacity(limit as _);
                {
                    let mut r = sqlx::query(&query_str).fetch(&mut *tx);
                    while let Some(row) = r.try_next().await? {
                        groups.push(self.from_row(row));
                    }
                }
                self.cached_groups_array
                    .insert(query_str, groups.clone())
                    .await;
                groups
            },
        )
    }

    pub async fn set_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        status: GroupStatus,
    ) -> Result<bool> {
        let r = sqlx::query("UPDATE groups SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        self.invalidate_cache(id).await;
        Ok(r.rows_affected() == 1)
    }

    pub async fn get_status(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<GroupStatus> {
        let r = sqlx::query("SELECT status FROM groups WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(r.get("status"))
    }
}
