use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::FromRow;

use fofo_utils::{usizedb, ContentType};
use user_system::model::UserType;

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum CategoryStatus {
    Active,
    Archived,
    Stopped,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
pub struct Category {
    pub id: usizedb,
    pub title: String,
    pub description: Option<String>,
    pub description_content_type: ContentType,
    pub status: CategoryStatus,
    pub read_level: UserType,
    pub write_level: UserType,
    pub comment_level: UserType,
    pub moderator_ids: Vec<usizedb>,
    pub group_ids: Vec<usizedb>,
    pub total_post: usizedb,
    pub cover_url: Option<String>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
pub struct CategoryToCreate {
    pub title: String,
    pub description: String,
    pub description_content_type: ContentType,
    pub status: CategoryStatus,
    pub read_level: UserType,
    pub write_level: UserType,
    pub comment_level: UserType,
    pub moderator_ids: Vec<usizedb>,
    pub group_ids: Vec<usizedb>,
    pub cover_url: Option<String>,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
pub struct CategoryToUpdate {
    pub title: String,
    pub description: String,
    pub description_content_type: ContentType,
    pub status: CategoryStatus,
    pub read_level: UserType,
    pub write_level: UserType,
    pub comment_level: UserType,
    pub moderator_ids: Vec<usizedb>,
    pub group_ids: Vec<usizedb>,
    pub cover_url: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum GetCategoriesSort {
    Id,
    Title,
    TotalPost,
}

#[derive(Debug, FromRow, Clone)]
pub(crate) struct CategoryLevels {
    pub read_level: UserType,
    pub write_level: UserType,
    pub comment_level: UserType,
}
