use actix_web::{delete, get, post, put, web, HttpResponse};

use crate::{
    api::{
        group_controller::model::{GetGroupsQuery, SetStatusBody},
        util::{GetDatasExtended, ListSlice, VerificationTargetWrapper},
        SDW,
    },
    request_client::RequestClient,
};
use fofo_utils::usizedb;
use group_system::model::{Group, GroupToCreateUpdate};

use super::api_error::ApiError;

mod model;

#[get("/group/{id}")]
pub async fn get_group(s: SDW, path: web::Path<(usizedb,)>) -> Result<HttpResponse, ApiError> {
    let (cid,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(false).await;
    let group = s.group.get_group(tx.as_mut(), cid).await?;
    match group {
        Some(group) => Ok(HttpResponse::Ok().json(group)),
        None => ApiError::no_group_found().to_err(),
    }
}

#[get("/groups")]
pub async fn get_groups(
    s: SDW,
    query: web::Query<GetGroupsQuery>,
) -> Result<web::Json<GetDatasExtended<Group>>, ApiError> {
    let _extended = query.extended;
    let desc = query.desc;
    let index = query.index as usizedb;
    let limit = query.limit as usizedb;

    if limit as usize > s.core.get_config().fetch_limit {
        return ApiError::fetch_limit().to_err();
    }

    let mut tx = s.core.begin_unwrap(false).await;
    let items = s
        .group
        .get_groups(tx.as_mut(), index, limit, query.into_inner().sort, desc)
        .await?;
    let total = s.group.get_count(tx.as_mut(), index, limit).await?;
    let mut data = GetDatasExtended::empty();
    data.set_data(ListSlice { items, total });
    Ok(web::Json(data))
}

#[post("/group")]
pub async fn create_group(
    s: SDW,
    to_create: web::Json<VerificationTargetWrapper<GroupToCreateUpdate>>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }
    let mut tx = s.core.begin_unwrap(true).await;
    if s.group
        .is_exists_duplicate_title(tx.as_mut(), &to_create.target.title, None)
        .await?
    {
        return ApiError::unique_group_required().to_err();
    }
    tx.commit_unwrap().await;
    let g = s.group.create_group(to_create.into_inner().target).await?;
    Ok(HttpResponse::Ok().json(g))
}

#[delete("/group/{id}")]
pub async fn delete_group(s: SDW, path: web::Path<(usizedb,)>) -> Result<HttpResponse, ApiError> {
    let (cid,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(true).await;
    if s.group.delete_group(tx.as_mut(), cid).await? {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_group_found().to_err()
    }
}

#[put("/group/{id}")]
pub async fn update_group(
    s: SDW,
    to_update: web::Json<VerificationTargetWrapper<GroupToCreateUpdate>>,
    path: web::Path<(usizedb,)>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }
    let mut tx = s.core.begin_unwrap(false).await;
    let (id,) = path.into_inner();

    if !s.group.is_exists(tx.as_mut(), id).await? {
        return ApiError::no_group_found().to_err();
    }
    if s.group
        .is_exists_duplicate_title(tx.as_mut(), &to_update.target.title, Some(id))
        .await?
    {
        return ApiError::unique_group_required().to_err();
    }
    tx.commit_unwrap().await;
    let mut tx = s.core.begin_unwrap(true).await;
    match s
        .group
        .update_group(tx.as_mut(), id, to_update.into_inner().target)
        .await?
    {
        Some(g) => {
            tx.commit().await.unwrap();
            Ok(HttpResponse::Ok().json(g))
        }
        None => ApiError::unique_group_required().to_err(),
    }
}

#[put("/group_status/{id}")]
pub async fn set_group_status(
    s: SDW,
    client: RequestClient,
    path: web::Path<(usizedb,)>,
    body: web::Json<SetStatusBody>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }

    let (id,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(true).await;

    if s.group
        .set_status(tx.as_mut(), id, body.into_inner().status)
        .await?
    {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_group_found().to_err()
    }
}
