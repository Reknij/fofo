mod model;

use actix_web::{
    get,
    http::StatusCode,
    put,
    web::{self, Json},
    HttpResponse,
};
use anyhow::Result;
use chrono::Utc;
use storage::{model::PresignedLocalStorageQuery, SimpleStorageService};

use crate::{api::{api_error::ApiError, storage_controller::model::{GetPresignedUrlQuery, GetPresignedUrlResult}, SDW}, request_client::RequestClient};


#[put("/upload_put_presigned/{key:.*}")]
pub async fn upload_presigned(
    s: SDW,
    client: RequestClient,
    path: web::Path<(String,)>,
    body: web::Bytes,
    query: web::Query<PresignedLocalStorageQuery>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !s.storage.is_local() {
        return ApiError::unsupported_api().to_err();
    }
    let (key,) = path.into_inner();
    let storage = s.storage.as_local();
    let mut tx = s.core.begin_unwrap(true).await;
    if !storage.verify(tx.as_mut(), &key, &query.authorize).await? {
        return ApiError::no_permission("Can't authorization! Upload failed.").to_err();
    }

    return if storage.put_object(&key, &body).await? {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::save_file_error().to_err()
    };
}

#[get("/static/{key:.*}")]
pub async fn get_static_file(s: SDW, path: web::Path<(String,)>) -> Result<HttpResponse, ApiError> {
    let (key,) = path.into_inner();
    let storage = s.storage.as_local();
    let bytes = storage.get_object(&key).await?;
    match bytes {
        Some(bytes) => {
            let resp = HttpResponse::build(StatusCode::OK).body(bytes);
            Ok(resp)
        }
        None => ApiError::no_static_file_found().to_err(),
    }
}

#[get("/presign_put_url")]
pub async fn presign_put_url(
    s: SDW,
    client: RequestClient,
    query: web::Query<GetPresignedUrlQuery>,
) -> Result<Json<GetPresignedUrlResult>, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    match query.signed_flag {
        model::SignedFlag::UserAvatar => (),
        model::SignedFlag::PostCover => {
            if !s.core.get_config().custom_post_cover_supported {
                return ApiError::unsupported_api().to_err();
            }
        }
        model::SignedFlag::CategoryCover => {
            if !client.get_user_unwrap().is_admin() {
                return ApiError::only_admin().to_err();
            }
        }
    }

    let current = client.get_user_unwrap();
    let ext: &str = std::path::Path::new(&query.filename)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    let now = Utc::now().timestamp_millis();
    let username_with_time = format!("{}-{}", &current.username, now);
    let hash = fofo_utils::calc_hash(&username_with_time);
    let key = format!("{hash}.{ext}");
    let presigned_url = s.storage
        .get_presign_put_url_marked(&key, query.into_inner().signed_flag.into(), 0, false)
        .await?;

    Ok(Json(GetPresignedUrlResult {
        presigned_url,
        object_url: s.storage.get_real_url(key),
    }))
}
