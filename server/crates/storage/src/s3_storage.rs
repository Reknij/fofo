use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use fofo_utils::usizedb;
use s3::{creds::Credentials, Bucket, Region};
use shared_core::SharedCore;
use sqlx::SqliteConnection;
use std::{fmt::Debug, sync::Arc};
use tokio::task::JoinHandle;

use super::{
    object_marker::{model::ObjectFlag, ObjectMarker},
    SimpleStorageService,
};

#[derive(Debug, Clone)]
pub struct S3 {
    bucket: Bucket,
    bucket_url: String,
    core: SharedCore,
    marker: ObjectMarker,
    check_permanent_handle: Option<Arc<JoinHandle<()>>>,
}

impl S3 {
    pub async fn new(core: SharedCore, marker: ObjectMarker) -> Self {
        let conf = core.get_config().clone();
        let s3 = conf
            .s3
            .clone()
            .expect("Please ensure the S3Config is not null");
        let bucket = Bucket::new(
            s3.bucket.as_ref().unwrap(),
            Region::Custom {
                region: s3.region.clone().unwrap().into_owned(),
                endpoint: s3.endpoint.clone().unwrap().into_owned(),
            },
            Credentials::default().unwrap(),
        )
        .unwrap()
        .with_path_style();
        let bucket_url = bucket.url();
        let mut this = S3 {
            bucket,
            bucket_url,
            core,
            marker,
            check_permanent_handle: None,
        };

        let t1 = this.clone();
        let resource_expiry_seconds = conf.resource_expiry_seconds;
        this.check_permanent_handle = Some(Arc::new(tokio::spawn(async move {
            loop {
                let earliest =
                    (Utc::now().timestamp() as usize - resource_expiry_seconds) as usizedb;
                let mut tx = t1.core.begin_unwrap(true).await;
                let objects = t1
                    .marker
                    .get_permanent_marked_objects(tx.as_mut(), false, 1000)
                    .await
                    .unwrap();
                tx.commit_unwrap().await;

                for object in objects {
                    if !object.permanent && object.created_at < earliest {
                        t1.delete_object(&object.key).await.unwrap();
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(resource_expiry_seconds as _))
                    .await;
            }
        })));

        this
    }
}
#[async_trait]
impl SimpleStorageService for S3 {
    async fn put_object(&self, key: &str, content: &[u8]) -> Result<bool> {
        let response_data = self.bucket.put_object(key, content).await?;
        Ok(response_data.status_code() == 200)
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
        let response_data = self.bucket.get_object(key).await?;
        let result = if response_data.status_code() == 200 {
            Some(response_data.bytes().clone())
        } else {
            None
        };
        Ok(result)
    }

    async fn delete_object(&self, key: &str) -> Result<bool> {
        let response_data = self.bucket.delete_object(key).await?;
        Ok(response_data.status_code() == 200)
    }

    async fn get_presign_put_url(&self, key: &str) -> Result<String> {
        let conf = self.core.get_config();
        let expiry_secs = conf.presign_expiry_seconds as _;
        let url = self.bucket.presign_put(key, expiry_secs, None)?;
        Ok(url)
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
        let conf = self.core.get_config();
        if key.starts_with("http://") || key.starts_with("https://") {
            key
        } else {
            let url = if let Some(url) = &conf.s3.as_ref().unwrap().public_url {
                url.as_ref()
            } else {
                &self.bucket_url
            };
            format!("{url}/{key}")
        }
    }

    fn try_parse_url_to_key<'a>(&self, url: &'a str) -> Option<&'a str> {
        let conf = self.core.get_config();
        let base = if let Some(url) = &conf.s3.as_ref().unwrap().public_url {
            url.as_ref()
        } else {
            &self.bucket_url
        };
        if url.starts_with(&base) {
            let start = base.len() + 1;
            let key = &url[start..];
            Some(key)
        } else {
            None
        }
    }
}
