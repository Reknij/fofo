use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use fofo_utils::{calc_hash, usizedb};

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum UserStatus {
    Active,
    Banned,
    OnlyComment,
    Observer,
}

#[derive(
    Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type, PartialOrd,
)]
#[repr(u8)]
pub enum UserType {
    Guest,
    General,
    Administrator,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct UserInfo {
    pub id: usizedb,
    pub email: String,
    pub username: String,
    pub alias: String,
    pub password: String,
    pub group_ids: Vec<usizedb>,
    pub status: UserStatus,
    pub user_type: UserType,
    pub avatar_url: Option<String>,
    pub signature: String,
    pub created_at: usizedb,
    pub total_post: usizedb,
    pub total_comment: usizedb,
}
#[derive(Debug, sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct SafeUserInfo {
    pub id: usizedb,
    pub alias: String,
    pub username: String,
    pub group_ids: Vec<usizedb>,
    pub status: UserStatus,
    pub user_type: UserType,
    pub avatar_url: Option<String>,
    pub signature: String,
    pub created_at: usizedb,
    pub total_post: usizedb,
    pub total_comment: usizedb,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserToCreate {
    pub email: String,
    pub username: String,
    pub password: String,
    pub alias: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserToUpdate {
    pub email: String,
    pub password: String,
    pub alias: String,
    pub avatar_url: Option<String>,
    pub signature: String,
}

impl UserInfo {
    pub fn generate_auth(&self) -> String {
        let now = Utc::now().timestamp();
        let content = format!("{}{}{}-{}", &self.id, &self.username, &self.password, now);
        calc_hash(&content).to_string()
    }

    pub fn is_admin(&self) -> bool {
        self.user_type == UserType::Administrator
    }
}

impl Into<SafeUserInfo> for UserInfo {
    fn into(self) -> SafeUserInfo {
        SafeUserInfo {
            id: self.id,
            alias: self.alias,
            username: self.username,
            group_ids: self.group_ids,
            status: self.status,
            user_type: self.user_type,
            avatar_url: self.avatar_url,
            signature: self.signature,
            created_at: self.created_at,
            total_post: self.total_post,
            total_comment: self.total_comment,
        }
    }
}
