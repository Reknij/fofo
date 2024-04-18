use serde::{Deserialize, Serialize};

use comment_system::model::{CommentStatus, GetCommentsSort};
use fofo_utils::usizedb;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCommentsQuery {
    pub post_id: usizedb,
    pub parent_id: usizedb,
    pub index: usizedb,
    pub limit: usizedb,
    pub sort: GetCommentsSort,
    pub desc: bool,
    #[serde(default)]
    pub extended: bool,
    #[serde(default = "GetCommentsQuery::enable")]
    pub top_order_enable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCommentLikeStatusQuery {
    pub comment_id: usizedb,
    pub user_id: usizedb,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetStatusBody {
    pub status: CommentStatus,
}

impl GetCommentsQuery {
    pub fn enable() -> bool {
        true
    }
}
