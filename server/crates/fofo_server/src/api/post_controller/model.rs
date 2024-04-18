use serde::{Deserialize, Serialize};

use crate::api::util::VerificationTargetWrapper;
use anyhow::Result;
use fofo_utils::usizedb;
use post_system::model::{PostAlgorithmOrder, PostStatus, PostToCreate, PostToUpdate};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStatusBody {
    pub status: PostStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostQuery {
    #[serde(default = "disabled")]
    pub full: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostsQuery {
    pub sort: PostAlgorithmOrder,
    #[serde(default)]
    pub distinct: bool,
    pub category_id: Option<usizedb>,
    pub created_by_id: Option<usizedb>,
    #[serde(default = "default_time_num")]
    pub time_num: usizedb,
    #[serde(default = "default_time")]
    pub time: String,
    pub index: usizedb,
    pub limit: usizedb,
    #[serde(default)]
    pub extended: bool,
    #[serde(default)]
    pub top_order_enable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostsCountQuery {
    pub sort: PostAlgorithmOrder,
    pub category_id: Option<usizedb>,
    pub created_by_id: Option<usizedb>,
    #[serde(default)]
    pub distinct: bool,
    #[serde(default = "default_time_num")]
    pub time_num: usizedb,
    #[serde(default = "default_time")]
    pub time: String,
}

fn default_time_num() -> usizedb {
    1
}

fn default_time() -> String {
    "lifetime".to_owned()
}

fn disabled() -> bool {
    false
}

macro_rules! impl_verify {
    ($($t:ty),+) => {
        $(impl crate::api::util::Verify<Result<(), crate::ApiError>> for $t {
            fn verify(&self)-> Result<(), crate::ApiError> {
                use crate::api::util::LegalityVerification;
                use crate::ApiError;
                let c = &self.target;
                if !LegalityVerification::is_title(&c.title) {
                    return ApiError::illegal_title().to_err()
                }

                if !LegalityVerification::is_content(&c.content) {
                    return ApiError::illegal_content().to_err()
                }

                if c.tags.len() > 9 {
                    return ApiError::too_many_tags().to_err();
                }

                for tag in &c.tags {
                    if !LegalityVerification::is_tag(tag) {
                        return ApiError::illegal_tag(tag).to_err();
                    }
                }
                Ok(())
            }
        })*
    };
}

impl_verify!(
    VerificationTargetWrapper<PostToCreate>,
    VerificationTargetWrapper<PostToUpdate>
);
