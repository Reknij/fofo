use std::{borrow::Cow, default::Default, env, path::Path, process::exit, sync::Arc};

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;
use tracing::{error, info};

pub type SafeConfig = Arc<Config>;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Buffer size. Sqlite maximum bulk insert is 999.
    pub buffer_size: usize,
    /// Request pagination maximum limit.
    pub fetch_limit: usize,
    /// All task trigger interval in millisecond.
    pub task_trigger_ms: usize,
    /// Users logined active duration in day.
    pub auth_active_days: usize,
    /// resources expiry duration in second.
    pub resource_expiry_seconds: usize,
    /// temporary resource expiry duration in second.
    pub temporary_expiry_seconds: usize,
    /// users get presign url expiry duration in second.
    pub presign_expiry_seconds: usize,
    /// All check task trigger interval in millisecond.
    pub check_task_interval_seconds: usize,
    /// The maximum capacity of entries that the cache can hold.
    pub cache_max_capacity: u64,
    /// ttl in second.
    pub ttl_seconds: u64,
    /// tti in second.
    pub tti_seconds: u64,
    /// Post editable duration in second.
    pub editable_seconds: u64,
    /// Post and comment top index maximum, ignore admin.
    pub top_index_max: u64,
    /// Auto fetch the post cover from article if post created cover is empty.
    pub auto_fetch_post_cover: bool,
    /// User can upload the post cover or not.
    pub custom_post_cover_supported: bool,
    /// User can register or not.
    pub open_register: bool,
    /// Local storage service config. (If S3 is disabled)
    pub local: LocalStorageConfig,
    /// Use API compatible with the Amazon S3 cloud storage service. Defaults to None, if defined it means enabled.
    pub s3: Option<S3Config>,
    /// Use forwarded ip instead peer ip. It use header `x-forwarded-for` to get the ip.
    pub forwarded_ip: bool,
    /// The bypass key for rate limit. If request with header `x-bypass-key` equals to this key will bypass it. Default is none.
    pub bypass_key: Option<Cow<'static, str>>,
    /// Console log level.
    pub log_level: Cow<'static, str>,
    /// Image format. Example, using when generate captcha. Go to https://docs.rs/image/latest/image/enum.ImageFormat.html see more.
    pub image_format: Cow<'static, str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    /// Bucket of Amazon S3 cloud storage service.
    pub bucket: Option<Cow<'static, str>>,
    /// Region of Amazon S3 cloud storage service.
    pub region: Option<Cow<'static, str>>,
    /// Endpoint of Amazon S3 cloud storage service.
    pub endpoint: Option<Cow<'static, str>>,
    /// Endpoint of Amazon S3 cloud storage service.
    pub public_url: Option<Cow<'static, str>>,
    /// Access Key ID of Amazon S3 cloud storage service.
    pub access_key: Option<Cow<'static, str>>,
    /// Secret Access Key of Amazon S3 cloud storage service.
    pub secret_key: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalStorageConfig {
    /// Public url of local storage.
    #[serde(default = "LocalStorageConfig::default_public_url")]
    pub public_url: Cow<'static, str>,
    /// Max bytes of client upload.
    #[serde(default = "LocalStorageConfig::default_max_bytes")]
    pub max_bytes: usize,
}

impl LocalStorageConfig {
    pub fn default_public_url() -> Cow<'static, str> {
        LocalStorageConfig::default().public_url
    }
    pub fn default_max_bytes() -> usize {
        LocalStorageConfig::default().max_bytes
    }
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            public_url: Cow::Borrowed(""),
            max_bytes: 2 * 1024 * 1024,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            buffer_size: 999,
            fetch_limit: 30,
            task_trigger_ms: 500,
            auth_active_days: 180,
            resource_expiry_seconds: 3600,
            temporary_expiry_seconds: 60,
            presign_expiry_seconds: 8,
            check_task_interval_seconds: 3600,
            cache_max_capacity: 1000,
            ttl_seconds: 5,
            tti_seconds: 5,
            editable_seconds: 30 * 60,
            top_index_max: 9,
            auto_fetch_post_cover: true,
            custom_post_cover_supported: false,
            open_register: true,
            local: LocalStorageConfig::default(),
            s3: None,
            bypass_key: None,
            forwarded_ip: false,
            log_level: "info".into(),
            image_format: "jpeg".into(),
        }
    }
}

impl Config {
    pub async fn new<P>(path: P) -> Result<SafeConfig>
    where
        P: AsRef<Path>,
    {
        let config = if path.as_ref().is_file() {
            match fs::read_to_string(path).await {
                Ok(v) => match toml::from_str::<Config>(&v) {
                    Ok(v) => v,
                    Err(err) => {
                        error!("Deserialize config occurs error:\n{}", err);
                        exit(0)
                    }
                },
                Err(err) => {
                    error!("Read config file occurs error:\n{}", err);
                    exit(0)
                }
            }
        } else {
            info!("Using default config.");
            Self::default()
        };

        if let Some(s3) = &config.s3 {
            const ACCESS_KEY: &str = "AWS_ACCESS_KEY_ID";
            const SECRET_KEY: &str = "AWS_SECRET_ACCESS_KEY";
            if let Some(v) = s3.access_key.as_ref() {
                env::set_var(ACCESS_KEY, v.as_ref())
            }
            if let Some(v) = s3.secret_key.as_ref() {
                env::set_var(SECRET_KEY, v.as_ref())
            }
            if s3.bucket.is_none()
                || s3.region.is_none()
                || env::var(ACCESS_KEY).is_err()
                || env::var(SECRET_KEY).is_err()
                || s3.endpoint.is_none()
                || s3.public_url.is_none()
            {
                error!("Enable `s3` but not provide all s3 configuration.");
                exit(0);
            }
        }

        Ok(Arc::new(config))
    }
}
