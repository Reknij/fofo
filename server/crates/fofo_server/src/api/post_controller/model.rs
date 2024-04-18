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
    pub sort: PostAlgorithmOrder,
    #[serde(default)]
    pub distinct: bool,
    pub category_id: Option<usizedb>,
    pub created_by_id: Option<usizedb>,
    #[serde(default = "GetPostQuery::default_time_num")]
    pub time_num: usizedb,
    #[serde(default = "GetPostQuery::default_time")]
    pub time: String,
    pub index: usizedb,
    pub limit: usizedb,
    #[serde(default)]
    pub extended: bool,
    #[serde(default = "GetPostQuery::enable")]
    pub top_order_enable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostCountQuery {
    pub sort: PostAlgorithmOrder,
    pub category_id: Option<usizedb>,
    pub created_by_id: Option<usizedb>,
    #[serde(default)]
    pub distinct: bool,
    #[serde(default = "GetPostQuery::default_time_num")]
    pub time_num: usizedb,
    #[serde(default = "GetPostQuery::default_time")]
    pub time: String,
}

impl GetPostQuery {
    pub fn default_time_num() -> usizedb {
        1
    }

    pub fn default_time() -> String {
        "lifetime".to_owned()
    }

    pub fn enable() -> bool {
        true
    }
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
