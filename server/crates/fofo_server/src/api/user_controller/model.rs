use anyhow::Result;
use crate::api::{
    api_error::ApiError,
    util::{LegalityVerification, VerificationTargetWrapper},
};
use fofo_utils::usizedb;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use user_system::model::{UserInfo, UserStatus, UserToCreate, UserToUpdate};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToDeleteUsers {
    pub ids: Vec<usizedb>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserById {
    pub id: usizedb,
    pub index: usizedb,
    pub limit: usizedb,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersQuery {
    pub sort: GetUsersSort,
    pub desc: bool,
    pub index: usizedb,
    pub limit: usizedb,
    #[serde(default = "GetUsersQuery::default_extended")]
    pub extended: bool,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum GetUsersSort {
    Id,
    Username,
    Alias,
    UserLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToLoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthAndUser {
    pub auth: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthQuery {
    pub auth: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStatusBody {
    pub status: UserStatus,
}

impl ToString for GetUsersSort {
    fn to_string(&self) -> String {
        match &self {
            GetUsersSort::Id => "id".to_owned(),
            GetUsersSort::Username => "username".to_owned(),
            GetUsersSort::Alias => "alias".to_owned(),
            GetUsersSort::UserLevel => "user_level".to_owned(),
        }
    }
}

impl GetUsersQuery {
    pub fn default_extended() -> bool {
        false
    }
}

impl VerificationTargetWrapper<UserToCreate> {
    pub fn verify(&self) -> Result<(), ApiError> {
        let u = &self.target;
        if !LegalityVerification::is_email(&u.email) {
            return ApiError::illegal_email().to_err();
        }
        if !LegalityVerification::is_username(&u.username) {
            return ApiError::illegal_username().to_err();
        }
        if !LegalityVerification::is_password(&u.password) {
            return ApiError::illegal_password().to_err();
        }

        Ok(())
    }
}

impl VerificationTargetWrapper<UserToUpdate> {
    pub fn verify(&self) -> Result<(), ApiError> {
        let u = &self.target;
        if !LegalityVerification::is_email(&u.email) {
            return ApiError::illegal_email().to_err();
        }
        if !LegalityVerification::is_password(&u.password) {
            return ApiError::illegal_password().to_err();
        }

        Ok(())
    }
}

impl VerificationTargetWrapper<ToLoginUser> {
    pub fn verify(&self) -> Result<(), ApiError> {
        let u = &self.target;
        if !LegalityVerification::is_username(&u.username) {
            return ApiError::illegal_username().to_err();
        }
        if !LegalityVerification::is_password(&u.password) {
            return ApiError::illegal_password().to_err();
        }

        Ok(())
    }
}
