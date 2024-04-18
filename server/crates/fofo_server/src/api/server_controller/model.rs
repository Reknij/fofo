use serde::{Deserialize, Serialize};

use fofo_utils::usizedb;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub editable_seconds: usizedb,
    pub open_register: bool,
    pub custom_post_cover_supported: bool,
    pub auth_active_days: usizedb,
}
