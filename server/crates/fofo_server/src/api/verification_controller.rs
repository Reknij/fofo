pub mod model;

use actix_web::{get, http::StatusCode, HttpRequest, HttpResponse};
use tracing::warn;

use crate::api::{api_error::{ApiError, DetailErrorCode}, SDW};

#[get("/verification")]
pub async fn get_verification(s: SDW, _req: HttpRequest) -> Result<HttpResponse, ApiError> {
    match s.verification.get_verification().await {
        Ok(v) => Ok(HttpResponse::Ok().json(v)),
        Err(err) => {
            warn!("Can't get captcha: {}", err);
            ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                DetailErrorCode::GetVerificationError,
                "Internal server can't get the captcha code and picture.",
            )
            .to_err()
        }
    }
}
