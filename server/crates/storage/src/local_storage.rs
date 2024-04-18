use channel_cache::ChannelCacheTask;
use shared_core::SharedCore;
use fofo_utils::usizedb;
use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use sqlx::{QueryBuilder, SqliteConnection};
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use tokio::{fs, io::AsyncWriteExt, task::JoinHandle};
use tracing::warn;

use super::{
    object_marker::{model::ObjectFlag, ObjectMarker},
    SimpleStorageService,
};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    static_path: PathBuf,
    presign_base_url: String,
    get_static_base_url: String,
    presign_channel: Option<ChannelCacheTask<String, String>>,
    core: SharedCore,
    marker: ObjectMarker,
    check_permanent_handle: Option<Arc<JoinHandle<()>>>,
    check_presign_handle: Option<Arc<JoinHandle<()>>>,
}

impl LocalStorage {
    pub async fn new(
        core: SharedCore,
        marker: ObjectMarker,
        get_api_path: &str,
        put_api_path: &str,
    ) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "
        CREATE TABLE IF NOT EXISTS presigned_urls(
            key TEXT PRIMARY KEY,
            authorize TEXT NOT NULL,
            expiry_secs INT NOT NULL,
            created_at INT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS presigned_urls_key_and_authorize_index
            on presigned_urls (key, authorize);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap(); // create indexes.

        tx.commit().await.unwrap();
        let meta = core.get_meta();
        let config = core.get_config();

        let cache_max_capacity = config.cache_max_capacity as usize;
        let task_trigger_ms = config.task_trigger_ms;
        let temporary_expiry_seconds = config.temporary_expiry_seconds;
        let resource_expiry_seconds = config.resource_expiry_seconds;
        let check_task_interval_seconds = config.check_task_interval_seconds as u64;

        let mut this = LocalStorage {
            core,
            static_path: meta.data_path.join("static"),
            presign_base_url: format!("{}{}", config.local.public_url, put_api_path),
            get_static_base_url: format!("{}{}", config.local.public_url, get_api_path),
            presign_channel: None,
            marker,
            check_permanent_handle: None,
            check_presign_handle: None,
        };

        let t1 = this.clone();
        this.presign_channel = Some(ChannelCacheTask::new(
            "get_presign_put_urls".into(),
            cache_max_capacity,
            task_trigger_ms,
            move |arr| {
                let t = t1.clone();
                async move {
                    let mut tx = t.core.begin_unwrap(true).await;
                    let r = t.get_presign_put_urls(tx.as_mut(), arr).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));

        let t1 = this.clone();
        this.check_permanent_handle = Some(Arc::new(tokio::spawn(async move {
            loop {
                let mut tx = t1.core.begin_unwrap(true).await;
                let objects = t1
                    .marker
                    .get_permanent_marked_objects(tx.as_mut(), false, 1000)
                    .await
                    .unwrap();
                tx.commit_unwrap().await;

                for object in objects {
                    let earliest = match object.flag {
                        ObjectFlag::Captcha => {
                            Utc::now().timestamp() as usize - temporary_expiry_seconds
                        }
                        _ => Utc::now().timestamp() as usize - resource_expiry_seconds,
                    } as usizedb;
                    if !object.permanent && object.created_at < earliest {
                        t1.delete_object(&object.key).await.unwrap();
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(check_task_interval_seconds))
                    .await;
            }
        })));

        let t1 = this.clone();
        this.check_presign_handle = Some(Arc::new(tokio::spawn(async move {
            loop {
                let now = Utc::now().timestamp();
                let mut tx = t1.core.begin_unwrap(true).await;
                sqlx::query("DELETE FROM presigned_urls WHERE created_at + expiry_secs < ?")
                    .bind(now)
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                tx.commit_unwrap().await;
                tokio::time::sleep(std::time::Duration::from_secs(check_task_interval_seconds))
                    .await;
            }
        })));

        this
    }

    pub async fn verify(
        &self,
        tx: &mut SqliteConnection,
        key: &str,
        authorize: &str,
    ) -> Result<bool> {
        let now = Utc::now().timestamp();
        let r = sqlx::query("DELETE FROM presigned_urls WHERE key = ? AND authorize = ? AND created_at + expiry_secs > ?")
            .bind(key)
            .bind(authorize)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        Ok(r.rows_affected() == 1)
    }

    async fn get_presign_put_urls(
        &self,
        tx: &mut SqliteConnection,
        vec: Vec<String>,
    ) -> Result<Vec<String>> {
        let now = Utc::now().timestamp();
        let mut processed = Vec::with_capacity(vec.len());
        let conf = self.core.get_config();
        let expiry_secs = conf.presign_expiry_seconds as usizedb;
        for key in vec {
            let authorize = fofo_utils::calc_hash(&format!("{now}+{key}"));
            processed.push((key, authorize.to_string()));
        }
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO presigned_urls(key, authorize, expiry_secs, created_at) ",
        );
        query_builder.push_values(&processed, |mut b, (key, authorize)| {
            b.push_bind(key)
                .push_bind(authorize)
                .push_bind(expiry_secs)
                .push_bind(now);
        });
        let r = query_builder.build().execute(&mut *tx).await?;
        if r.rows_affected() == processed.len() as u64 {
            let arr = processed
                .into_iter()
                .map(|(key, authorize)| {
                    format!("{}/{key}?authorize={authorize}", &self.presign_base_url)
                })
                .collect();
            Ok(arr)
        } else {
            bail!("Get presign_put_urls failed!")
        }
    }
}

#[async_trait]
impl SimpleStorageService for LocalStorage {
    async fn put_object(&self, key: &str, content: &[u8]) -> Result<bool> {
        let path = self.static_path.join(key);
        if path.exists() {
            bail!("Key already exists!")
        }
        if let Some(path) = path.parent() {
            fs::create_dir_all(path).await?;
        }
        let mut fs = fs::File::create(path).await?;
        fs.write(content).await?;
        Ok(true)
    }

    async fn put_object_marked(
        &self,
        key: String,
        content: &[u8],
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<bool> {
        let r = self.put_object(&key, content).await;
        let this = self.clone();
        tokio::spawn(async move {
            this.marker
                .mark(key, flag, ref_id, permanent)
                .await
                .expect("mark object failed.");
        });
        r
    }

    async fn get_object(&self, key: &str) -> Result<Option<Bytes>> {
        let path = self.static_path.join(key);
        Ok(if path.exists() {
            let bytes = fs::read(&path).await?;
            Some(bytes.into())
        } else {
            None
        })
    }

    async fn delete_object(&self, key: &str) -> Result<bool> {
        let path = self.static_path.join(key);
        Ok(if path.exists() {
            fs::remove_file(path).await?;
            true
        } else {
            false
        })
    }

    async fn get_presign_put_url(&self, key: &str) -> Result<String> {
        let key = key.to_owned();
        match self.presign_channel.as_ref() {
            Some(task) => {
                let r = task.send(key).await?;
                Ok(r)
            }
            None => bail!("Don't have the task."),
        }
    }

    async fn get_presign_put_url_marked(
        &self,
        key: &str,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<String> {
        let r = self.get_presign_put_url(&key).await;
        let this = self.clone();
        let key = key.to_owned();
        tokio::spawn(async move {
            this.marker
                .mark(key, flag, ref_id, permanent)
                .await
                .expect("mark object failed.");
        });
        r
    }

    async fn remark(
        &self,
        tx: &mut SqliteConnection,
        key: &str,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<bool> {
        let r = self
            .marker
            .remark(&mut *tx, key, flag, ref_id, permanent)
            .await;
        r
    }

    fn get_real_url(&self, key: String) -> String {
        if key.starts_with("https://") || key.starts_with("http://") {
            key
        } else {
            format!("{}/{key}", &self.get_static_base_url)
        }
    }

    fn try_parse_url_to_key<'a>(&self, url: &'a str) -> Option<&'a str> {
        warn!(
            "Original url: {url}, to gsbu: {}",
            &self.get_static_base_url
        );
        if url.starts_with(&self.get_static_base_url) {
            let start = self.get_static_base_url.len() + 1;
            let key = &url[start..];
            Some(key)
        } else {
            None
        }
    }
}
