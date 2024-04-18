use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use fofo_utils::usizedb;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserNotificationArguments {
    pub ref_id: usizedb,
    pub target_user_id: usizedb,
    pub n_type: UserNotificationType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserNotification {
    pub id: usizedb,
    pub ref_id: usizedb,
    pub target_user_id: usizedb,
    pub created_by_id: usizedb,
    pub created_at: usizedb,
    pub n_type: UserNotificationType,
    pub readed: bool,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum UserNotificationType {
    Comment,
    ReplyComment,
    LikePost,
    DislikePost,
    LikeComment,
    DislikeComment,
}
