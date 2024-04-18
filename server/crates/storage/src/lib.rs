use std::ops::Deref;

use self::{
    local_storage::LocalStorage,
    object_marker::{model::ObjectFlag, ObjectMarker},
    s3_storage::S3,
};
use shared_core::SharedCore;
use fofo_utils::usizedb;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use sqlx::SqliteConnection;

pub mod local_storage;
pub mod model;
pub mod object_marker;
pub mod s3_storage;

#[async_trait]
pub trait SimpleStorageService: std::fmt::Debug + Send + Sync {
    async fn put_object(&self, key: &str, content: &[u8]) -> Result<bool>;
    async fn put_object_marked(
        &self,
        key: String,
        content: &[u8],
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<bool>;

    async fn get_object(&self, key: &str) -> Result<Option<Bytes>>;
    async fn delete_object(&self, key: &str) -> Result<bool>;

    async fn get_presign_put_url(&self, key: &str) -> Result<String>;
    async fn get_presign_put_url_marked(
        &self,
        key: &str,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<String>;

    fn get_real_url(&self, key: String) -> String;
    fn try_parse_url_to_key<'a>(&self, url: &'a str) -> Option<&'a str>;
    async fn remark(
        &self,
        tx: &mut SqliteConnection,
        key: &str,
        flag: ObjectFlag,
        ref_id: usizedb,
        permanent: bool,
    ) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub enum S3Ref {
    S3(S3),
    LocalStorage(LocalStorage),
}

impl S3Ref {
    pub async fn new(core: SharedCore) -> S3Ref {
        let object_marker = ObjectMarker::new(core.clone()).await;
        if core.get_config().s3.is_some() {
            S3Ref::S3(S3::new(core.clone(), object_marker).await)
        } else {
            S3Ref::LocalStorage(
                LocalStorage::new(
                    core,
                    object_marker,
                    "/api/static",
                    "/api/upload_put_presigned",
                )
                .await,
            )
        }
    }

    pub fn is_local(&self) -> bool {
        match self {
            S3Ref::LocalStorage(_) => true,
            _ => false,
        }
    }

    pub fn as_local(&self) -> &LocalStorage {
        match self {
            S3Ref::LocalStorage(v) => v,
            _ => panic!("Is not local!"),
        }
    }
}

impl Deref for S3Ref {
    type Target = dyn SimpleStorageService;

    fn deref(&self) -> &Self::Target {
        match self {
            S3Ref::S3(v) => v,
            S3Ref::LocalStorage(v) => v,
        }
    }
}
