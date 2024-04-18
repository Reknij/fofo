use fofo_utils::usizedb;
use serde::{Deserialize, Serialize};

use category_system::model::{
    CategoryStatus, CategoryToCreate, CategoryToUpdate, GetCategoriesSort,
};
use crate::api::{
    api_error::ApiError,
    util::{LegalityVerification, VerificationTargetWrapper, Verify},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCategoriesQuery {
    pub index: usizedb,
    pub limit: usizedb,
    pub sort: GetCategoriesSort,
    pub desc: bool,
    #[serde(default = "GetCategoriesQuery::default_extended")]
    pub extended: bool,
}

impl GetCategoriesQuery {
    pub fn default_extended() -> bool {
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStatusBody {
    pub status: CategoryStatus,
}

macro_rules! impl_verify {
    ($($t:ty),+) => {
        $(impl Verify<Result<(), ApiError>> for $t {
            fn verify(&self)-> Result<(), ApiError> {
                let c = &self.target;
                if !LegalityVerification::is_title(&c.title) {
                    return ApiError::illegal_title().to_err()
                }

                if !LegalityVerification::is_content(&c.description) {
                    return ApiError::illegal_content().to_err()
                }
                Ok(())
            }
        })*
    };
}

impl_verify!(
    VerificationTargetWrapper<CategoryToCreate>,
    VerificationTargetWrapper<CategoryToUpdate>
);
