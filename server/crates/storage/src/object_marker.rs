pub mod model;

use anyhow::{bail, Result};
use channel_cache::ChannelCacheTask;
use chrono::Utc;
use fofo_utils::usizedb;
use futures::TryStreamExt;
use shared_core::SharedCore;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};

use self::model::{MarkedObject, ObjectFlag};

#[derive(Debug, Clone)]
pub struct ObjectMarker {
    core: SharedCore,
    mark_channel: Option<ChannelCacheTask<(String, ObjectFlag, usizedb, bool), ()>>,
}

impl ObjectMarker {
    pub async fn new(core: SharedCore) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS marked_objects(
            key TEXT PRIMARY KEY,
            flag INT NOT NULL,
            flag_ref_id INT NOT NULL,
            permanent BOOLEAN NOT NULL,
            created_at INT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
        let config = core.get_config().clone();
        let mut this = ObjectMarker {
            core: core.clone(),
            mark_channel: None,
        };
        let t1 = this.clone();
        this.mark_channel = Some(ChannelCacheTask::new(
            "mark_objects".into(),
            config.cache_max_capacity as _,
            config.task_trigger_ms as _,
            move |arr| {
                let t = t1.clone();
                async move {
                    let mut tx = t.core.begin_unwrap(true).await;
                    let r = t
                        .mark_many(tx.as_mut(), arr)
                        .await
                        .unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));

        this
    }

    pub async fn mark(
        &self,
        key: String,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<()> {
        match self.mark_channel.as_ref() {
            Some(task) => {
                let r = task.send((key, flag, ref_id, permanent)).await?;
                Ok(r)
            }
            None => bail!("Don't have the task."),
        }
    }

    async fn mark_many(
        &self,
        tx: &mut SqliteConnection,
        vec: Vec<(String, ObjectFlag, usizedb, bool)>,
    ) -> Result<Vec<()>> {
        let now = Utc::now().timestamp();
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO marked_objects (key, flag, flag_ref_id, permanent, created_at) ",
        );
        query_builder.push_values(&vec, |mut b, (key, flag, ref_id, permanent)| {
            b.push_bind(key)
                .push_bind(flag)
                .push_bind(ref_id)
                .push_bind(permanent)
                .push_bind(now);
        });
        let r = query_builder.build().execute(&mut *tx).await?;

        if r.rows_affected() == vec.len() as u64 {
            let mut arr = Vec::with_capacity(vec.len());
            for _ in 0..vec.len() {
                arr.push(());
            }

            Ok(arr)
        } else {
            bail!("Insert objects marked failed.")
        }
    }

    pub async fn remark(
        &self,
        tx: &mut SqliteConnection,
        key: &str,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<bool> {
        let r = sqlx::query(
            "UPDATE marked_objects SET flag = ?, flag_ref_id = ?, permanent = ? WHERE key = ? AND (flag_ref_id = 0 OR (flag = ? AND flag_ref_id = ?))",
        )
        .bind(flag)
        .bind(ref_id)
        .bind(permanent)
        .bind(key)
        .bind(flag)
        .bind(ref_id)
        .execute(&mut *tx)
        .await?;
        Ok(r.rows_affected() == 1)
    }

    fn from_row(&self, row: SqliteRow) -> MarkedObject {
        MarkedObject {
            key: row.try_get("key").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            flag: row.try_get("flag").unwrap(),
            flag_ref_id: row.try_get("flag_ref_id").unwrap(),
            permanent: row.try_get("permanent").unwrap(),
        }
    }

    pub async fn get_permanent_marked_objects(
        &self,
        tx: &mut SqliteConnection,
        permanent: bool,
        limit: usizedb,
    ) -> Result<Vec<MarkedObject>> {
        let mut rows = sqlx::query("SELECT * FROM marked_objects WHERE permanent = ? LIMIT ?")
            .bind(permanent)
            .bind(limit)
            .fetch(&mut *tx);
        let mut arr = Vec::with_capacity(limit as _);
        while let Some(row) = rows.try_next().await? {
            arr.push(self.from_row(row))
        }
        Ok(arr)
    }

    #[allow(dead_code)]
    pub async fn get_marked_object(
        &self,
        tx: &mut SqliteConnection,
        key: &str,
    ) -> Result<Option<MarkedObject>> {
        let mut r = sqlx::query("SELECT * FROM marked_objects WHERE key = ? LIMIT 1")
            .bind(key)
            .fetch(&mut *tx);
        Ok(if let Some(row) = r.try_next().await? {
            let object = self.from_row(row);
            Some(object)
        } else {
            None
        })
    }
}
