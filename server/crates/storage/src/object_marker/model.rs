use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use fofo_utils::usizedb;

#[derive(
    Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type, PartialOrd,
)]
#[repr(u8)]
pub enum ObjectFlag {
    Captcha,
    UserAvatar,
    PostCover,
    CategoryCover,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MarkedObject {
    pub key: String,
    pub created_at: usizedb,
    pub flag: ObjectFlag,
    pub flag_ref_id: usizedb,
    pub permanent: bool,
}