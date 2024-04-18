use std::sync::Arc;

use anyhow::{bail, Result};
use channel_cache::ChannelCacheTask;
use chrono::{Duration, Utc};
use fofo_utils::usizedb;
use shared_core::SharedCore;
use storage::S3Ref;

use futures::TryStreamExt;
use moka::future::Cache;
use sqlx::sqlite::SqliteRow;
use sqlx::{QueryBuilder, Row, SqliteConnection};

use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use self::model::*;

pub mod model;

#[derive(Debug, Clone)]
pub struct UserSystem {
    core: SharedCore,
    s3: S3Ref,
    cached_users: Cache<usizedb, UserInfo>,
    cached_users_array: Cache<String, Vec<UserInfo>>,
    create_task: Option<ChannelCacheTask<(UserToCreate, UserStatus, UserType), UserInfo>>,
    check_task: Option<Arc<JoinHandle<()>>>,
}

impl UserSystem {
    pub async fn new(core: SharedCore, s3: S3Ref) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        let require_admin = !fofo_utils::exists_table(tx.as_mut(), "users").await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users(
                id INTEGER PRIMARY KEY,
                username varchar(128) NOT NULL,
                alias varchar(128) NOT NULL,
                email varchar(128) NOT NULL,
                password varchar(128) NOT NULL,
                user_type INT NOT NULL,
                status INT NOT NULL,
                signature TEXT NOT NULL,
                created_at INT NOT NULL,
                avatar_url TEXT NULL,
                total_post INT NOT NULL,
                total_comment INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_groups(
            user_id INT NOT NULL,
            group_id INT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS auth_users(
                auth PRIMARY KEY,
                user_id INT NOT NULL,
                created_at INT NULL,

                FOREIGN KEY(user_id) REFERENCES users(id)
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS user_username_index
            on users (username);
            CREATE UNIQUE INDEX IF NOT EXISTS user_email_index
            on users (email);
            CREATE UNIQUE INDEX IF NOT EXISTS user_email_index
            on users (email);
            CREATE INDEX IF NOT EXISTS user_groups_user
            on user_groups (user_id);
            CREATE INDEX IF NOT EXISTS user_groups_group
            on user_groups (group_id);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        let config = core.get_config();
        let mut this = UserSystem {
            s3,
            core: core.clone(),
            cached_users: fofo_utils::get_cache_instance(config.clone()).await,
            cached_users_array: fofo_utils::get_cache_instance(config.clone()).await,
            create_task: None,
            check_task: None,
        };

        if require_admin {
            let username = "admin".to_owned();
            let password = "admin123".to_owned();
            let admin = UserToCreate {
                alias: "Admin".to_owned(),
                email: "admin@email.com".to_owned(),
                username: username.to_owned(),
                password: password.to_owned(),
            };
            this.create_users(
                tx.as_mut(),
                vec![(admin, UserStatus::Active, UserType::Administrator)],
            )
            .await
            .unwrap();
            info!("Admin created, username: {username}, password: {password}");
        }

        tx.commit().await.unwrap();
        let us = this.clone();
        let us2 = this.clone();
        this.create_task = Some(ChannelCacheTask::new(
            "create_users".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |utcs| {
                let us = us.clone();
                async move {
                    let mut tx = us.core.begin_unwrap(true).await;
                    let r = us.create_users(tx.as_mut(), utcs).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));

        this.check_task = Some(Arc::new(tokio::spawn(async move {
            let mut tx = us2.core.begin_unwrap(true).await;
            us2.check_auths_expired(tx.as_mut()).await.unwrap();
            tx.commit_unwrap().await;
            tokio::time::sleep(std::time::Duration::from_secs(60 * 2)).await;
        })));

        this
    }

    pub async fn is_exists(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM users WHERE id=? LIMIT 1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some())
    }

    pub async fn create_user(
        &self,
        user: UserToCreate,
        status: UserStatus,
        user_type: UserType,
    ) -> Result<UserInfo> {
        match self.create_task.as_ref() {
            Some(task) => {
                let user = task.send((user, status, user_type)).await?;
                Ok(user)
            }
            None => bail!("Don't have the task."),
        }
    }

    async fn insert_groups(
        &self,
        tx: &mut SqliteConnection,
        values: Vec<(usizedb, Vec<usizedb>)>,
    ) -> Result<()> {
        let ids = values
            .iter()
            .map(|(id, _)| id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        sqlx::query(&format!(
            "DELETE FROM user_groups WHERE user_id IN ({ids});"
        ))
        .execute(&mut *tx)
        .await?;
        QueryBuilder::new("INSERT INTO user_groups (user_id, group_id) ")
            .push_values(&values, |mut b, (user_id, groups)| {
                for group_id in groups {
                    b.push_bind(user_id).push_bind(group_id);
                }
            })
            .build()
            .execute(&mut *tx)
            .await?;
        Ok(())
    }

    async fn create_users(
        &self,
        tx: &mut SqliteConnection,
        users: Vec<(UserToCreate, UserStatus, UserType)>,
    ) -> Result<Vec<UserInfo>> {
        let len = users.len() as usizedb;
        let now = Utc::now().timestamp() as usizedb;
        let signature = "Hello everyone, I am part of the forum.".to_owned();
        let mut query_builder = QueryBuilder::new("INSERT INTO users (username, alias, password, email, status, user_type, signature, created_at, total_post, total_comment) ");
        query_builder.push_values(&users, |mut b, (user, user_status, user_type)| {
            b.push_bind(&user.username)
                .push_bind(&user.alias)
                .push_bind(&user.password)
                .push_bind(&user.email)
                .push_bind(user_status)
                .push_bind(user_type)
                .push_bind(&signature)
                .push_bind(now)
                .push_bind(0)
                .push_bind(0);
        });
        let query = query_builder.build();
        let r = query.execute(&mut *tx).await?;

        if r.rows_affected() == (len as u64) {
            let base_id = r.last_insert_rowid() as usizedb - len + 1;
            let arr: Vec<_> = users
                .into_iter()
                .enumerate()
                .map(|(i, (user, user_status, user_type))| UserInfo {
                    id: base_id + i as usizedb,
                    username: user.username,
                    email: user.email,
                    alias: user.alias,
                    password: user.password,
                    group_ids: vec![1], // default group id is 1
                    status: user_status,
                    user_type,
                    avatar_url: None,
                    signature: signature.to_owned(),
                    created_at: now,
                    total_post: 0,
                    total_comment: 0,
                })
                .collect();

            let groups = arr.iter().map(|u| (u.id, u.group_ids.to_owned())).collect();
            self.insert_groups(tx, groups).await?;
            Ok(arr)
        } else {
            bail!("Insert failed.")
        }
    }

    pub async fn update_user(
        &self,
        tx: &mut SqliteConnection,
        user_id: usizedb,
        mut user: UserToUpdate,
    ) -> Result<Option<UserInfo>> {
        if let Some(url) = &user.avatar_url {
            if let Some(key) = self.s3.try_parse_url_to_key(url) {
                if url != key {
                    user.avatar_url = Some(key.to_owned())
                }
            }
        }
        let r = sqlx::query(
            "UPDATE users SET alias=?, password=?, email=?, avatar_url=?, signature=? WHERE id = ?",
        )
        .bind(&user.alias)
        .bind(&user.password)
        .bind(&user.email)
        .bind(&user.avatar_url)
        .bind(&user.signature)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() > 1 {
            error!("rows affected is more than 1!");
        }

        Ok(if r.rows_affected() == 1 {
            self.invalidate_cache(user_id).await;
            let user = self.get_user(tx, user_id).await?;
            user
        } else {
            None
        })
    }

    async fn invalidate_cache(&self, id: usizedb) {
        self.cached_users.invalidate(&id).await;
        self.cached_users_array
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

    pub async fn get_user(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<Option<UserInfo>> {
        Ok(if let Some(cached) = self.cached_users.get(&id) {
            Some(cached)
        } else {
            let r = sqlx::query("SELECT * FROM users WHERE id=? LIMIT 1")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?;
            if let Some(row) = r {
                let v = self.from_row(tx, row).await;
                self.cached_users.insert(id, v.clone()).await;
                Some(v)
            } else {
                None
            }
        })
    }

    pub async fn get_safe_user(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
    ) -> Result<Option<SafeUserInfo>> {
        Ok(if let Some(cached) = self.cached_users.get(&id) {
            Some(cached.into())
        } else {
            let r = sqlx::query("SELECT * FROM users WHERE id=? LIMIT 1")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?;
            if let Some(row) = r {
                let v = self.from_row(tx, row).await;
                self.cached_users.insert(id, v.clone()).await;
                Some(v.into())
            } else {
                None
            }
        })
    }

    pub async fn get_user_by_username(
        &self,
        tx: &mut SqliteConnection,
        username: &str,
    ) -> Result<Option<UserInfo>> {
        let r = sqlx::query("SELECT * FROM users WHERE username=? LIMIT 1")
            .bind(username)
            .fetch_optional(&mut *tx)
            .await?;
        Ok(if let Some(row) = r {
            Some(self.from_row(tx, row).await)
        } else {
            None
        })
    }

    pub async fn get_user_by_email(
        &self,
        tx: &mut SqliteConnection,
        email: &str,
    ) -> Result<Option<UserInfo>> {
        let r = sqlx::query("SELECT * FROM users WHERE email=? LIMIT 1")
            .bind(email)
            .fetch_optional(&mut *tx)
            .await?;
        Ok(if let Some(row) = r {
            Some(self.from_row(tx, row).await)
        } else {
            None
        })
    }

    async fn from_row(&self, tx: &mut SqliteConnection, row: SqliteRow) -> UserInfo {
        let avatar_url = {
            let cover_url: Option<String> = row.get("avatar_url");
            cover_url.map(|url| self.s3.get_real_url(url))
        };
        let id = row.try_get("id").unwrap();
        UserInfo {
            id,
            username: row.try_get("username").unwrap(),
            password: row.try_get("password").unwrap(),
            alias: row.try_get("alias").unwrap(),
            group_ids: sqlx::query("SELECT group_id as id FROM user_groups WHERE user_id=?")
                .bind(id)
                .fetch_all(&mut *tx)
                .await
                .map(|rows| rows.into_iter().map(|row| row.get("id")).collect())
                .unwrap_or(Vec::new()),
            email: row.try_get("email").unwrap(),
            status: row.try_get("status").unwrap(),
            user_type: row.try_get("user_type").unwrap(),
            avatar_url,
            signature: row.try_get("signature").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            total_post: row.try_get("total_post").unwrap(),
            total_comment: row.try_get("total_comment").unwrap(),
        }
    }

    pub async fn get_users(&self, tx: &mut SqliteConnection, cond: &str) -> Result<Vec<UserInfo>> {
        let q = format!("SELECT * FROM users {}", cond);

        Ok(if let Some(cached) = self.cached_users_array.get(&q) {
            cached
        } else {
            let r = sqlx::query(cond).fetch_all(&mut *tx).await?;
            let mut users = Vec::with_capacity(r.len());
            for row in r {
                users.push(self.from_row(tx, row).await);
            }
            self.cached_users_array.insert(q, users.clone()).await;
            users
        })
    }

    pub async fn get_safe_users(
        &self,
        tx: &mut SqliteConnection,
        cond: &str,
    ) -> Result<Vec<SafeUserInfo>> {
        let users = self.get_users(tx, cond).await?;
        Ok(users.into_iter().map(|u: UserInfo| u.into()).collect())
    }

    pub async fn delete_user(&self, tx: &mut SqliteConnection, id: usizedb) -> Result<bool> {
        let r = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if r.rows_affected() > 1 {
            warn!("rows affected is more than 1.")
        }
        Ok(r.rows_affected() == 1)
    }

    pub async fn check_auths_expired(&self, tx: &mut SqliteConnection) -> Result<()> {
        let config = self.core.get_config();
        sqlx::query(&format!("DELETE FROM auth_users WHERE created_at < CAST (strftime ('%s', datetime ('now', '-{} days')) AS INT)", config.auth_active_days)).execute(&mut *tx).await?;
        Ok(())
    }

    pub async fn get_and_save_auth(
        &self,
        tx: &mut SqliteConnection,
        user: &UserInfo,
    ) -> Result<String> {
        let auth = user.generate_auth();
        let now = Utc::now().timestamp();
        let r = sqlx::query("INSERT INTO auth_users (auth, user_id, created_at) VALUES (?,?,?);")
            .bind(&auth)
            .bind(user.id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() == 1 {
            Ok(auth)
        } else {
            bail!("Insert auth failed.")
        }
    }

    pub async fn remove_auth(&self, tx: &mut SqliteConnection, auth: &str) -> Result<bool> {
        let r = sqlx::query("DELETE FROM auth_users WHERE auth = ?")
            .bind(auth)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn revert(&self, auth: &str) -> Result<Option<UserInfo>> {
        let mut tx = self.core.begin_unwrap(false).await;

        let auth_active_days = Duration::days(self.core.get_config().auth_active_days as i64);
        let auth_row = sqlx::query("SELECT * FROM auth_users WHERE auth = ?")
            .bind(auth)
            .fetch_one(tx.as_mut())
            .await;
        match auth_row {
            Ok(r) => {
                let created_at: usizedb = r.try_get("created_at").unwrap();
                if created_at <= fofo_utils::get_last_active_time(auth_active_days).await {
                    info!("Authorization of user is expired..");
                    return Ok(None);
                }
                let user_id = r.try_get("user_id").unwrap();

                Ok(self.get_user(tx.as_mut(), user_id).await?)
            }
            _ => Ok(None),
        }
    }

    pub async fn get_count(&self, _index: usizedb, limit: usizedb) -> Result<usizedb> {
        Ok(limit * 100) // Because counting is too slow. Return directly 100 pages.
    }

    pub async fn set_status(
        &self,
        tx: &mut SqliteConnection,
        id: usizedb,
        status: UserStatus,
    ) -> Result<bool> {
        let r = sqlx::query("UPDATE users SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        self.invalidate_cache(id).await;
        Ok(r.rows_affected() == 1)
    }
}
