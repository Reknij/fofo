use crate::{
    api::{
        api_error::ApiError,
        category_controller::model::{GetCategoriesQuery, SetStatusBody},
        util::{
            check_category, GetDatasExtended, GetDatasExtendedBuilder, ListSlice,
            VerificationTargetWrapper, Verify, WhatToDo,
        },
    },
    request_client::RequestClient,
};
use actix_web::{delete, get, post, put, web, HttpResponse};
use category_system::model::{Category, CategoryToCreate, CategoryToUpdate};
use fofo_utils::usizedb;
use storage::object_marker::model::ObjectFlag;

use super::SDW;

pub mod model;

#[get("/category/{id}")]
pub async fn get_category(
    s: SDW,
    path: web::Path<(u32,)>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    let (cid,) = path.into_inner();
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    check_category(&s, tx.as_mut(), cid, user, WhatToDo::None).await?;
    let v = s.category.get_category(tx.as_mut(), cid).await?.unwrap();
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(v))
}

#[get("/categories")]
pub async fn get_categories(
    s: SDW,
    query: web::Query<GetCategoriesQuery>,
) -> Result<web::Json<GetDatasExtended<Category>>, ApiError> {
    let extended = query.extended;
    let desc = query.desc;
    let index = query.index as usizedb;
    let limit = query.limit as usizedb;
    if limit as usize > s.core.get_config().fetch_limit {
        return ApiError::fetch_limit().to_err();
    }
    let mut tx = s.core.begin_unwrap(false).await;
    let items = s
        .category
        .get_categories(
            tx.as_mut(),
            query.index as _,
            query.limit as _,
            query.into_inner().sort,
            desc,
        )
        .await?;
    let total = s.category.get_count(tx.as_mut(), index, limit).await?;
    let mut builder = GetDatasExtendedBuilder::new(&s);
    if extended {
        for c in &items {
            builder
                .extend_categories(tx.as_mut(), c.group_ids.to_owned())
                .await?;
            builder
                .extend_users(tx.as_mut(), c.moderator_ids.to_owned())
                .await?;
        }
    }
    let data = builder.set_data(ListSlice { items, total }).build();
    Ok(web::Json(data))
}

#[post("/category")]
pub async fn create_category(
    s: SDW,
    to_create: web::Json<VerificationTargetWrapper<CategoryToCreate>>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }
    let mut tx = s.core.begin_unwrap(false).await;
    to_create.verify()?;
    if s.category
        .is_exists_duplicate_title(tx.as_mut(), &to_create.target.title, None)
        .await?
    {
        return ApiError::unique_category_required().to_err();
    }
    for id in &to_create.target.group_ids {
        if s.group.is_exists(tx.as_mut(), id.to_owned()).await? {
            return ApiError::no_group_found().to_err();
        }
    }

    for id in &to_create.target.moderator_ids {
        if !s.user.is_exists(tx.as_mut(), id.to_owned()).await? {
            return ApiError::no_user_found().to_err();
        }
    }
    tx.commit_unwrap().await;

    if let Some(cover_url) = &to_create.target.cover_url {
        // change newest cover permanent to true to keep it permanent.
        if let Some(key) = s.storage.try_parse_url_to_key(&cover_url) {
            let mut tx = s.core.begin_unwrap(true).await;
            s.storage
                .remark(
                    tx.as_mut(),
                    key,
                    ObjectFlag::CategoryCover,
                    client.get_user_unwrap().id,
                    true,
                )
                .await?;
            tx.commit_unwrap().await;
        }
    }

    let c = s
        .category
        .create_category(to_create.into_inner().target)
        .await?;
    Ok(HttpResponse::Ok().json(c))
}

#[delete("/category/{id}")]
pub async fn delete_category(s: SDW, path: web::Path<(u32,)>) -> Result<HttpResponse, ApiError> {
    let (cid,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(true).await;
    if s.category.delete_category(tx.as_mut(), cid).await? {
        tx.commit().await.expect("Commit failed!");
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_category_found().to_err()
    }
}

#[put("/category_status/{id}")]
pub async fn set_category_status(
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
    if s.category
        .set_status(tx.as_mut(), id, body.into_inner().status)
        .await?
    {
        tx.commit().await.expect("Commit failed!");
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_category_found().to_err()
    }
}

#[put("/category/{id}")]
pub async fn update_category(
    s: SDW,
    path: web::Path<(u32,)>,
    to_update: web::Json<VerificationTargetWrapper<CategoryToUpdate>>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }

    to_update.verify()?;
    let (id,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(false).await;
    if !s.category.is_exists(tx.as_mut(), id).await? {
        return ApiError::no_category_found().to_err();
    }
    if s.category
        .is_exists_duplicate_title(tx.as_mut(), &to_update.target.title, Some(id))
        .await?
    {
        return ApiError::unique_category_required().to_err();
    }

    for id in &to_update.target.group_ids {
        if s.group.is_exists(tx.as_mut(), id.to_owned()).await? {
            return ApiError::no_group_found().to_err();
        }
    }

    for id in &to_update.target.moderator_ids {
        if s.user.is_exists(tx.as_mut(), id.to_owned()).await? {
            return ApiError::no_user_found().to_err();
        }
    }
    let previous = s.category.get_category(tx.as_mut(), id).await?.unwrap();
    tx.commit_unwrap().await;

    // change old cover permanent to false to clear resource if not equal.
    if previous.cover_url != to_update.target.cover_url {
        let mut tx = s.core.begin_unwrap(true).await;
        if let Some(old_cover) = previous.cover_url {
            if let Some(key) = s.storage.try_parse_url_to_key(&old_cover) {
                s.storage
                    .remark(
                        tx.as_mut(),
                        key,
                        ObjectFlag::CategoryCover,
                        client.get_user_unwrap().id,
                        false,
                    )
                    .await?;
            }
        }
        if let Some(cover_url) = &to_update.target.cover_url {
            // change newest cover permanent to true to keep it permanent.
            if let Some(key) = s.storage.try_parse_url_to_key(&cover_url) {
                s.storage
                    .remark(
                        tx.as_mut(),
                        key,
                        ObjectFlag::CategoryCover,
                        client.get_user_unwrap().id,
                        true,
                    )
                    .await?;
            }
        }
        tx.commit_unwrap().await;
    }

    let mut tx = s.core.begin_unwrap(true).await;
    match s
        .category
        .update_category(tx.as_mut(), id, to_update.into_inner().target)
        .await?
    {
        Some(c) => {
            tx.commit_unwrap().await;
            Ok(HttpResponse::Ok().json(c))
        }
        None => ApiError::unique_category_required().to_err(),
    }
}
