use serde::{Serialize, Deserialize};
use sqlx::FromRow;

use fofo_utils::usizedb;

#[derive(Debug, FromRow)]
pub struct Verification {
    pub id: usizedb,
    pub created_at: usizedb,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerificationKeyPicture {
    pub verification_id: usizedb,
    pub secret_key_picture_url: String,
}