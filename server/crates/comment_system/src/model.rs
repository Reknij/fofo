use serde::{Serialize, Deserialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::FromRow;

use fofo_utils::{ContentType, usizedb};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CommentInfo {
    pub id: usizedb,
    pub parent_id: usizedb,
    pub created_at: usizedb,
    pub created_by_id: usizedb,
    pub last_edit_at: usizedb,
    pub last_edit_by_id: usizedb,
    pub content: String,
    pub content_type: ContentType,
    pub post_id: usizedb,
    pub category_id: usizedb,
    pub reply_user_id: usizedb,
    pub reply_comment_id: usizedb,
    pub likes: usizedb,
    pub dislikes: usizedb,
    pub status: CommentStatus,
    pub total_comment: usizedb,
    pub last_comment_at: usizedb,
    pub last_comment_by_id: usizedb,
    pub top_index: usizedb,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CommentBaseInfo {
    pub id: usizedb,
    pub parent_id: usizedb,
    pub created_at: usizedb,
    pub created_by_id: usizedb,
    pub last_edit_at: usizedb,
    pub last_edit_by_id: usizedb,
    pub content_type: ContentType,
    pub post_id: usizedb,
    pub category_id: usizedb,
    pub reply_user_id: usizedb,
    pub reply_comment_id: usizedb,
    pub likes: usizedb,
    pub dislikes: usizedb,
    pub status: CommentStatus,
    pub total_comment: usizedb,
    pub last_comment_at: usizedb,
    pub last_comment_by_id: usizedb,
    pub top_index: usizedb,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum CommentStatus {
    Active,
    Banned,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CommentToCreate {
    pub content: String,
    pub content_type: ContentType,
    pub post_id: usizedb,
    #[serde(default)]
    pub category_id: usizedb,
    #[serde(default)]
    pub parent_id: usizedb,
    #[serde(default)]
    pub top_index: usizedb,
    #[serde(default)]
    pub reply_user_id: usizedb,
    pub reply_comment_id: usizedb,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CommentToUpdate {
    #[serde(default)]
    pub top_index: usizedb,
    pub content: String,
    pub content_type: ContentType,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CommentLikeStatus {
    pub comment_id: usizedb,
    pub created_by_id: usizedb,
    pub created_at: usizedb,
    pub is_like: bool
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum GetCommentsSort {
    Id,
    Likes,
    Dislikes,
    TotalPost,
}