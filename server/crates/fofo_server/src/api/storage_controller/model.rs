use serde::{Serialize, Deserialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use storage::object_marker::model::ObjectFlag;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum SignedFlag {
    UserAvatar,
    PostCover,
    CategoryCover,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPresignedUrlQuery {
    pub signed_flag: SignedFlag,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPresignedUrlResult {
    pub object_url: String,
    pub presigned_url: String,
}

impl Into<ObjectFlag> for SignedFlag {
    fn into(self) -> ObjectFlag {
        match self {
            SignedFlag::UserAvatar => ObjectFlag::UserAvatar,
            SignedFlag::PostCover => ObjectFlag::PostCover,
            SignedFlag::CategoryCover => ObjectFlag::CategoryCover,
        }
    }
}