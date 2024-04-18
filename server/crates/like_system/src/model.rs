use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use sqlx::FromRow;

use fofo_utils::usizedb;


#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct LikeStatus {
    pub flag: LikeStatusFlag,
    pub flag_ref_id: usizedb,
    pub created_by_id: usizedb,
    pub created_at: usizedb,
    pub is_like: bool,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LikeStatusFlag {
    TargetPost,
    TargetComment,
}