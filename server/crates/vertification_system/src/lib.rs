mod model;
use anyhow::{bail, Result};

use captcha_rs::CaptchaBuilder;
use channel_cache::ChannelCacheTask;
use image::ImageFormat;
use shared_core::SharedCore;
use fofo_utils::usizedb;
use std::{borrow::Cow, io::Cursor};

use chrono::TimeZone;

use sqlx::{QueryBuilder, SqliteConnection};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{error, warn};

use storage::{object_marker::model::ObjectFlag, S3Ref};

use self::model::{Verification, VerificationKeyPicture};

#[derive(Debug, Clone)]
pub struct VerificationSystem {
    core: SharedCore,
    s3: S3Ref,
    image_format: ImageFormat,
    image_format_extension: Cow<'static, str>,
    check_task: Option<Arc<JoinHandle<()>>>,
    create_task: Option<ChannelCacheTask<(), VerificationKeyPicture>>,
}

impl VerificationSystem {
    pub async fn new(core: SharedCore, s3: S3Ref) -> Self {
        let mut tx = core.begin_unwrap(true).await;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS verifications(
                id INTEGER PRIMARY KEY,
                secret varchar(128) NOT NULL,
                created_at INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
        let config = core.get_config();
        let mut this = VerificationSystem {
            s3,
            image_format: ImageFormat::from_extension(core.get_config().image_format.as_ref()).unwrap_or(ImageFormat::WebP),
            image_format_extension: core.get_config().image_format.clone(),
            check_task: None,
            create_task: None,
            core,
        };

        let vs = this.clone();
        let vs2 = this.clone();
        let handle = tokio::spawn(async move {
            let mut tx = vs.core.begin_unwrap(true).await;
            vs.check_verifications_expired(tx.as_mut()).await.unwrap();
            tx.commit_unwrap().await;
            tokio::time::sleep(std::time::Duration::from_secs(60 * 2)).await;
        });
        this.check_task = Some(Arc::new(handle));
        this.create_task = Some(ChannelCacheTask::new(
            "get_verifications".into(),
            config.buffer_size,
            config.task_trigger_ms,
            move |b| {
                let len = b.len() as usizedb;
                let vs = vs2.clone();
                async move {
                    let mut tx = vs.core.begin_unwrap(true).await;
                    let r = vs.get_verifications(tx.as_mut(), len).await.unwrap();
                    tx.commit_unwrap().await;
                    r
                }
            },
        ));
        this
    }

    pub async fn check_verifications_expired(&self, tx: &mut SqliteConnection) -> Result<()> {
        let config = self.core.get_config();
        sqlx::query(&format!("DELETE FROM verifications WHERE created_at < CAST (strftime ('%s', datetime ('now', '-{} seconds')) AS INT)", config.temporary_expiry_seconds)).execute(&mut *tx).await?;
        Ok(())
    }

    pub async fn get_verification(&self) -> Result<VerificationKeyPicture> {
        match self.create_task.as_ref() {
            Some(task) => {
                let post = task.send(()).await?;
                Ok(post)
            }
            None => bail!("Don't have the task."),
        }
    }

    pub async fn get_verifications(
        &self,
        tx: &mut SqliteConnection,
        length: usizedb,
    ) -> Result<Vec<VerificationKeyPicture>> {
        let mut cs = Vec::with_capacity(length as _);
        for _ in 0..length {
            let captcha = CaptchaBuilder::new()
                .length(5)
                .width(130)
                .height(40)
                .dark_mode(false)
                .complexity(6) // min: 1, max: 10
                .build();
            cs.push(captcha)
        }

        let now = chrono::Utc::now().timestamp();
        let mut query_builder =
            QueryBuilder::new("INSERT INTO verifications (secret, created_at) ");
        query_builder.push_values(&cs, |mut b, captcha| {
            b.push_bind(&captcha.text).push_bind(&now);
        });
        let query = query_builder.build();
        let r = query.execute(&mut *tx).await?;

        if r.rows_affected() == length as u64 {
            let base_id = r.last_insert_rowid() as usizedb - length + 1;
            let mut vkps = Vec::with_capacity(length as _);
            for (i, captcha) in cs.into_iter().enumerate() {
                let id = base_id + i as usizedb;

                let mut content_bytes = Vec::with_capacity(captcha.image.as_bytes().len());
                captcha.image.write_to(&mut Cursor::new(&mut content_bytes), self.image_format)?;
                let key = {
                    let hash = fofo_utils::calc_hash(&content_bytes);
                    let key = format!("captcha/{hash}.{}", self.image_format_extension.as_ref());
                    if self.s3
                        .put_object_marked(
                            key.clone(),
                            &content_bytes,
                            ObjectFlag::Captcha,
                            0,
                            false,
                        )
                        .await?
                    {
                        key
                    } else {
                        error!("Can't store the captcha picture.");
                        "err_store_captcha".to_owned()
                    }
                };
                vkps.push(VerificationKeyPicture {
                    secret_key_picture_url: self.s3.get_real_url(key),
                    verification_id: id,
                });
            }
            Ok(vkps)
        } else {
            bail!("Insert failed!");
        }
    }

    pub async fn set_verification_passed(
        &self,
        tx: &mut SqliteConnection,
        verification_id: usizedb,
        secret_key: &str,
    ) -> Result<Option<bool>> {
        let r =
            sqlx::query_as::<_, Verification>("SELECT * FROM verifications WHERE id = ? LIMIT 1")
                .bind(verification_id)
                .fetch_optional(&mut *tx)
                .await?;
        Ok(match r {
            Some(v) => {
                let config = self.core.get_config();
                let now = chrono::Utc::now();
                let created_at = chrono::Utc.timestamp_opt(v.created_at as _, 0).unwrap();
                let diff = now.signed_duration_since(created_at);
                if v.secret == secret_key
                    && diff.num_seconds() < config.temporary_expiry_seconds as _
                {
                    let removed = self.remove_verification(&mut *tx, verification_id).await?;
                    if !removed {
                        warn!("Can't remove the verification from table.");
                    }
                    Some(true)
                } else {
                    Some(false)
                }
            }
            None => None,
        })
    }

    async fn remove_verification(
        &self,
        tx: &mut SqliteConnection,
        verification_id: usizedb,
    ) -> Result<bool> {
        let r = sqlx::query("DELETE FROM verifications WHERE id = ?")
            .bind(verification_id)
            .execute(&mut *tx)
            .await?;
        Ok(r.rows_affected() == 1)
    }
}
