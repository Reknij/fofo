use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use fofo_utils::{ContentType, usizedb};

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct Group {
    pub id: usizedb,
    pub title: String,
    pub description: String,
    pub description_content_type: ContentType,
    pub status: GroupStatus,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize, Clone)]
pub struct GroupToCreateUpdate {
    pub title: String,
    pub description: String,
    pub description_content_type: ContentType,
    pub status: GroupStatus,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum GroupStatus {
    Active,
    OnlyComment,
    Observer,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum GetGroupsSort {
    Id,
    Title,
}