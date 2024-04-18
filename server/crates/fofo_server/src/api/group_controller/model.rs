use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use group_system::model::{GetGroupsSort, GroupStatus, GroupToCreateUpdate};
use crate::api::util::VerificationTargetWrapper;
use fofo_utils::usizedb;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetGroupsQuery {
    pub index: usizedb,
    pub limit: usizedb,
    pub sort: GetGroupsSort,
    pub desc: bool,
    #[serde(default = "GetGroupsQuery::default_extended")]
    pub extended: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStatusBody {
    pub status: GroupStatus,
}

impl GetGroupsQuery {
    pub fn default_extended()-> bool {
        false
    }
}

macro_rules! impl_verify {
    ($($t:ty),+) => {
        $(impl crate::api::util::Verify<Result<(), crate::ApiError>> for $t {
            fn verify(&self)-> Result<(), crate::ApiError> {
                use crate::api::util::LegalityVerification;
                let c = &self.target;
                if !LegalityVerification::is_title(&c.title) {
                    return crate::ApiError::illegal_title().to_err()
                }

                if !LegalityVerification::is_content(&c.description) {
                    return crate::ApiError::illegal_content().to_err()
                }
                Ok(())
            }
        })*
    };
}

impl_verify!(VerificationTargetWrapper<GroupToCreateUpdate>);