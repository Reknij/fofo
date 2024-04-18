use channel_cache::ChannelCacheTask;
use fofo_utils::usizedb;
use shared_core::SharedCore;
use anyhow::{bail, Result};
use chrono::Utc;
use futures::TryStreamExt;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};

use self::model::{UserNotification, UserNotificationArguments, UserNotificationType};

pub mod model;

#[derive(Debug, Clone)]
pub struct NotificationSystem {
    core: SharedCore,
    create_task: Option<ChannelCacheTask<(usizedb, UserNotificationArguments), ()>>,
    set_readed: Option<ChannelCacheTask<(usizedb, usizedb, bool), ()>>,
}

impl NotificationSystem {
    pub async fn new(core: SharedCore) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_notifications(
                id INTEGER PRIMARY KEY,
                ref_id INT NOT NULL,
                created_by_id INT,
                target_user_id INT NOT NULL,
                created_at INT NOT NULL,
                n_type INT NOT NULL,
                readed BOOLEAN NOT NULL,

                FOREIGN KEY(created_by_id) REFERENCES users(id),
                FOREIGN KEY(target_user_id) REFERENCES users(id)
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
        let config = core.get_config();
        let mut this = NotificationSystem {
            core,
            create_task: None,
            set_readed: None,
        };
        let t2 = this.clone();
        let t3 = this.clone();
        let _t4 = this.clone();
        this.create_task = Some(ChannelCacheTask::new(
            "create_notifications".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |uns| {
                let t = t2.clone();
                async move {
                    let mut tx = t.core.begin_unwrap(true).await;
                    let r = t.create_user_notifications(tx.as_mut(), uns).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));
        this.set_readed = Some(ChannelCacheTask::new(
            "set_notifications_readed".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |uns| {
                let t = t3.clone();
                async move {
                    let mut tx = t.core.begin_unwrap(true).await;
                    let r = t
                        .set_user_notifications_read_status(tx.as_mut(), uns)
                        .await
                        .unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));

        this
    }

    pub async fn delete_user_notification(
        &self,
        tx: &mut SqliteConnection,
        user_id: usizedb,
        notification: UserNotificationArguments,
    ) -> Result<bool> {
        let r = sqlx::query("DELETE FROM user_notifications WHERE created_by_id=? AND ref_id=? AND n_type=? AND target_user_id=?").bind(user_id).bind(notification.ref_id).bind(notification.n_type).bind(notification.target_user_id).execute(&mut *tx).await?;
        Ok(r.rows_affected() == 1)
    }

    pub async fn create_user_notification(
        &self,
        user_id: usizedb,
        notification: UserNotificationArguments,
    ) -> Result<()> {
        if user_id == notification.target_user_id {
            return Ok(());
        }
        match self.create_task.as_ref() {
            Some(task) => {
                let delete_type = match notification.n_type {
                    UserNotificationType::LikePost => Some(UserNotificationType::DislikePost),
                    UserNotificationType::DislikePost => Some(UserNotificationType::LikePost),
                    UserNotificationType::LikeComment => Some(UserNotificationType::DislikeComment),
                    UserNotificationType::DislikeComment => Some(UserNotificationType::LikeComment),
                    _ => None,
                };
                if let Some(delete_type) = delete_type {
                    let this = self.clone();
                    tokio::spawn(async move {
                        let mut tx = this.core.begin_unwrap(true).await;
                        this.delete_user_notification(
                            tx.as_mut(),
                            user_id,
                            UserNotificationArguments {
                                n_type: delete_type,
                                ..notification
                            },
                        )
                        .await
                        .unwrap();
                        tx.commit_unwrap().await;
                    });
                }
                let n = task.send((user_id, notification)).await?;
                Ok(n)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn create_user_notifications(
        &self,
        tx: &mut SqliteConnection,
        notifications: Vec<(usizedb, UserNotificationArguments)>,
    ) -> Result<Vec<()>> {
        let len = notifications.len() as usizedb;
        let now = Utc::now().timestamp();
        let mut query_builder = QueryBuilder::new("INSERT INTO user_notifications (ref_id, created_by_id, target_user_id, created_at, n_type, readed) ");
        query_builder.push_values(&notifications, |mut b, (user_id, n)| {
            b.push_bind(&n.ref_id)
                .push_bind(user_id)
                .push_bind(&n.target_user_id)
                .push_bind(now)
                .push_bind(&n.n_type)
                .push_bind(false);
        });
        let query = query_builder.build();
        let r = query.execute(&mut *tx).await?;

        if r.rows_affected() == (len as u64) {
            let mut uns = Vec::with_capacity(len as _);
            for _ in 0..len {
                uns.push(())
            }
            Ok(uns)
        } else {
            bail!("Insert failed.")
        }
    }

    async fn from_row_to_user(&self, row: SqliteRow) -> UserNotification {
        UserNotification {
            id: row.try_get("id").unwrap(),
            created_by_id: row.try_get("created_by_id").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            ref_id: row.try_get("ref_id").unwrap(),
            target_user_id: row.try_get("target_user_id").unwrap(),
            n_type: row.try_get("n_type").unwrap(),
            readed: row.try_get("readed").unwrap(),
        }
    }

    pub async fn get_user_notifications(
        &self,
        tx: &mut SqliteConnection,
        user_id: usizedb,
        index: usizedb,
        limit: usizedb,
        only_unread: bool,
    ) -> Result<Vec<UserNotification>> {
        let offset = index * limit;
        let unread_q = if only_unread { "AND readed=FALSE" } else { "" };
        let q = format!("SELECT * FROM user_notifications WHERE target_user_id=? {unread_q} ORDER BY created_at DESC LIMIT ? OFFSET ?");
        let mut rows = sqlx::query(&q)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch(&mut *tx);
        let mut arr = Vec::with_capacity(limit as _);
        while let Some(row) = rows.try_next().await? {
            arr.push(self.from_row_to_user(row).await)
        }

        Ok(arr)
    }

    pub async fn get_user_notification_count(
        &self,
        tx: &mut SqliteConnection,
        user_id: usizedb,
        only_unread: bool,
    ) -> Result<usizedb> {
        let query = if only_unread {
            "SELECT COUNT(*) FROM user_notifications WHERE target_user_id=? AND readed=FALSE"
        } else {
            "SELECT COUNT(*) FROM user_notifications WHERE target_user_id=?"
        };
        let count: usizedb = sqlx::query_scalar(query)
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(count)
    }

    pub async fn set_user_notification_read_status(
        &self,
        user_id: usizedb,
        n_id: usizedb,
        readed: bool,
    ) -> Result<()> {
        match self.set_readed.as_ref() {
            Some(task) => {
                let r = task.send((user_id, n_id, readed)).await?;
                Ok(r)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn set_user_notifications_read_status(
        &self,
        tx: &mut SqliteConnection,
        vec: Vec<(usizedb, usizedb, bool)>,
    ) -> Result<Vec<()>> {
        let len = vec.len() as u64;
        let mut rows_affected = 0;
        for (user_id, n_id, readed) in vec {
            let query = if n_id > 0 {
                sqlx::query(
                    "UPDATE user_notifications SET readed=? WHERE id=? AND target_user_id=?",
                )
                .bind(readed)
                .bind(n_id)
                .bind(user_id)
            } else {
                sqlx::query("UPDATE user_notifications SET readed=? WHERE target_user_id=?")
                    .bind(readed)
                    .bind(user_id)
            };
            let r = query.execute(&mut *tx).await?;
            if r.rows_affected() > 0 {
                rows_affected += 1;
            }
        }

        if rows_affected == len {
            let mut vec = Vec::with_capacity(len as _);
            for _ in 0..len {
                vec.push(());
            }
            Ok(vec)
        } else {
            bail!("Update failed!")
        }
    }
}
