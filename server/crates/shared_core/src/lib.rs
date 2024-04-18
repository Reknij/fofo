mod db;

use std::ops::{Deref, DerefMut};

use anyhow::Result;
use fofo_utils::{config::SafeConfig, meta::SafeMetaInfo};
use sqlx::{Pool, Sqlite, Transaction};
use tracing::info;

#[derive(Debug, Clone)]
pub struct SharedCore {
    pool: Pool<Sqlite>,
    config: SafeConfig,
    meta: SafeMetaInfo,
}

pub struct TransactionWrapper<'a, DB>
where
    DB: sqlx::Database,
{
    tx: Option<Transaction<'a, DB>>,
}

impl<'a, DB> Deref for TransactionWrapper<'a, DB>
where
    DB: sqlx::Database,
{
    type Target = Transaction<'a, DB>;
    fn deref(&self) -> &Self::Target {
        self.tx.as_ref().expect("Commit or rollback already!")
    }
}

impl<'a, DB> DerefMut for TransactionWrapper<'a, DB>
where
    DB: sqlx::Database,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx.as_mut().expect("Commit or rollback already!")
    }
}

impl<'a, DB> Drop for TransactionWrapper<'a, DB>
where
    DB: sqlx::Database,
{
    fn drop(&mut self) {
        if self.tx.is_some() {
            self.tx.take();
        }
    }
}

impl<'a, DB> TransactionWrapper<'a, DB>
where
    DB: sqlx::Database,
{
    pub fn new(tx: Transaction<'a, DB>) -> Self {
        Self { tx: Some(tx) }
    }

    pub async fn commit(mut self) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
        }
        Ok(())
    }

    pub async fn commit_unwrap(mut self) {
        if let Some(tx) = self.tx.take() {
            tx.commit().await.expect("Transaction commit failed!");
        }
    }
    pub async fn rollback(mut self) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
        }
        Ok(())
    }
}

impl SharedCore {
    pub async fn new(config: SafeConfig, meta: SafeMetaInfo) -> Self {
        let pool = db::initial(
            meta.clone(),
            if cfg!(debug_assertions) { true } else { false },
        )
        .await
        .expect("Initial database failed!");
    
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS shared_core (
            id INTEGER PRIMARY KEY,
                reserved INT NOT NULL DEFAULT 1
            );
            INSERT OR IGNORE INTO shared_core (id, reserved) VALUES (1, 1);",
        )
        .execute(&pool)
        .await
        .unwrap();
        Self { pool, config, meta }
    }

    pub async fn begin(&self) -> Result<TransactionWrapper<'static, Sqlite>> {
        Ok(TransactionWrapper::new(self.pool.begin().await?))
    }

    pub async fn begin_unwrap(&self, to_write: bool) -> TransactionWrapper<'static, Sqlite> {
        let now = std::time::Instant::now();
        let mut r =
            TransactionWrapper::new(self.pool.begin().await.expect("Begin transaction failed!"));
        if to_write {
            sqlx::query("UPDATE shared_core SET reserved=1 WHERE id=1")
                .execute(r.as_mut())
                .await
                .expect("Try set transaction reserved manually failed!");
        }
        info!(
            "Begin transaction usage {:?}",
            std::time::Instant::now() - now
        );
        r
    }

    pub fn get_config(&self) -> SafeConfig {
        self.config.clone()
    }

    pub fn get_meta(&self) -> SafeMetaInfo {
        self.meta.clone()
    }
}
