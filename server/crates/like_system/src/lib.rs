use std::collections::HashMap;

use anyhow::{bail, Result};
use chrono::Utc;
use sqlx::{QueryBuilder, SqliteConnection};

use channel_cache::ChannelCacheTask;
use fofo_utils::usizedb;
use shared_core::SharedCore;

use model::{LikeStatus, LikeStatusFlag};

pub mod model;

#[derive(Debug, Clone)]
pub struct LikeSystem {
    core: SharedCore,
    like_status_task: Option<ChannelCacheTask<SetLikeStatusBundle, ()>>,
}

#[derive(Debug, Clone)]
struct SetLikeStatusBundle {
    flag_ref_id: usizedb,
    flag: LikeStatusFlag,
    user_id: usizedb,
    is_like: Option<bool>,
}

impl LikeSystem {
    pub async fn new(core: SharedCore) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS all_like_status(
                id INTEGER PRIMARY KEY,
                flag_ref_id INT NOT NULL,
                flag INT NOT NULL,
                created_at INT NOT NULL,
                created_by_id INT NOT NULL,
                is_like BOOLEAN NOT NULL,

                FOREIGN KEY(created_by_id) REFERENCES users(id)
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS like_status_m1_index
            on all_like_status (flag_ref_id, flag, created_by_id);
            CREATE INDEX IF NOT EXISTS like_status_m2_index
            on all_like_status (flag_ref_id, flag, created_by_id, is_like);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        tx.commit().await.unwrap();
        let config = core.get_config();
        let mut this = LikeSystem {
            core,
            like_status_task: None,
        };

        let t = this.clone();
        this.like_status_task = Some(ChannelCacheTask::new(
            "set_like_statuses".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |arr| {
                let t = t.clone();
                async move {
                    let mut tx = t.core.begin_unwrap(true).await;
                    let r = t.set_like_status_all(tx.as_mut(), arr).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));

        this
    }

    pub async fn get_like_status(
        &self,
        tx: &mut SqliteConnection,
        flag_ref_id: usizedb,
        flag: LikeStatusFlag,
        user_id: usizedb,
    ) -> Result<Option<LikeStatus>> {
        Ok(sqlx::query_as::<_, LikeStatus>(
            "SELECT * FROM all_like_status WHERE flag_ref_id = ? AND flag = ? AND created_by_id = ? LIMIT 1",
        )
        .bind(flag_ref_id)
        .bind(flag)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?)
    }

    pub async fn set_like_status(
        &self,
        flag_ref_id: usizedb,
        flag: LikeStatusFlag,
        user_id: usizedb,
        is_like: Option<bool>,
    ) -> Result<()> {
        match self.like_status_task.as_ref() {
            Some(task) => {
                let like_status = task
                    .send(SetLikeStatusBundle {
                        flag_ref_id,
                        flag,
                        user_id,
                        is_like,
                    })
                    .await?;
                Ok(like_status)
            }
            None => bail!("Don't have the task."),
        }
    }

    // The function to set the post likes status
    async fn set_like_status_all(
        &self,
        tx: &mut SqliteConnection,
        mut arr: Vec<SetLikeStatusBundle>,
    ) -> Result<Vec<()>> {
        let list = {
            let mut list = Vec::with_capacity(arr.len());
            for _ in 0..arr.len() {
                list.push(())
            }
            list
        };

        let mut like_statuses = Vec::with_capacity(arr.len());
        let bundles_by_map = {
            // Check if array exists data is same action from user.
            arr.reverse();
            let mut map = HashMap::with_capacity(arr.len());
            for bundle in arr {
                map.entry((bundle.flag, bundle.flag_ref_id, bundle.user_id))
                    .and_modify(|v| *v = bundle.clone())
                    .or_insert(bundle);
            }
            let bundles: Vec<SetLikeStatusBundle> =
                map.into_iter().map(|(_key, bundle)| bundle).collect();
            let mut map = HashMap::with_capacity(bundles.len());
            for b in bundles {
                map.entry((b.flag, b.flag_ref_id))
                    .and_modify(|v: &mut Vec<SetLikeStatusBundle>| v.push(b.clone()))
                    .or_insert(vec![b]);
            }
            map
        };

        let now = Utc::now().timestamp() as usizedb;
        for ((flag, flag_ref_id), bundles) in bundles_by_map {
            let ids = bundles
                .iter()
                .map(|b| b.user_id.to_string())
                .collect::<Vec<String>>()
                .join(",");
            let likes_deleted = sqlx::query(&format!("DELETE FROM all_like_status WHERE flag=? AND flag_ref_id=? AND created_by_id IN ({ids}) AND is_like=1")).bind(flag).bind(flag_ref_id).execute(&mut *tx).await?.rows_affected();
            let dislikes_deleted = sqlx::query(&format!("DELETE FROM all_like_status WHERE flag=? AND flag_ref_id=? AND created_by_id IN ({ids}) AND is_like=0")).bind(flag).bind(flag_ref_id).execute(&mut *tx).await?.rows_affected();
            let mut total_like = 0; // like of this bundle effect to likes_map;
            let mut total_dislike = 0; // dislike of this bundle effect to dislikes_map;

            for bundle in bundles {
                if let Some(is_like) = bundle.is_like {
                    if is_like {
                        total_like += 1
                    } else {
                        total_dislike += 1;
                    }

                    like_statuses.push(LikeStatus {
                        flag: bundle.flag,
                        flag_ref_id: bundle.flag_ref_id,
                        created_at: now,
                        created_by_id: bundle.user_id.to_owned(),
                        is_like: is_like.to_owned(),
                    })
                }
            }
            let total_like_insert = total_like - likes_deleted as i64;
            let total_dislike_insert = total_dislike - dislikes_deleted as i64;
            match flag {
                LikeStatusFlag::TargetPost => {
                    fofo_utils::increment_post_like(
                            tx,
                            flag_ref_id,
                            total_like_insert,
                            total_dislike_insert,
                        )
                        .await?;
                }
                LikeStatusFlag::TargetComment => {
                    fofo_utils::increment_comment_like(
                            tx,
                            flag_ref_id,
                            total_like_insert,
                            total_dislike_insert,
                        )
                        .await?;
                }
            }
        }

        if like_statuses.len() > 0 {
            QueryBuilder::new(
                "INSERT INTO all_like_status (flag_ref_id, flag, created_at, created_by_id, is_like) ",
            )
            .push_values(like_statuses, |mut b, bundle| {
                b.push_bind(bundle.flag_ref_id)
                    .push_bind(bundle.flag)
                    .push_bind(bundle.created_at)
                    .push_bind(bundle.created_by_id)
                    .push_bind(bundle.is_like);
            })
            .build()
            .execute(&mut *tx)
            .await?;
        }

        // Return Ok if no errors occurred
        Ok(list)
    }
}
