use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresignedLocalStorageQuery {
    pub authorize: String,
}