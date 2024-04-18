mod model;

use crate::{
    api::{
        api_error::ApiError,
        post_controller::model::{GetPostQuery, GetPostsQuery, SetStatusBody},
        util::{
            can_manage_post, check_category, check_post, check_user, GetDatasExtended,
            GetDatasExtendedBuilder, ListSlice, VerificationTargetWrapper, Verify,
        },
    },
    request_client::RequestClient,
};
use fofo_utils::usizedb;
use post_system::model::{PostFilterTime, PostInfo, PostToCreate, PostToUpdate};
use storage::object_marker::model::ObjectFlag;

use actix_web::{get, post, put, web, HttpResponse};

use super::{util::WhatToDo, SDW};

#[post("/post")]
pub async fn create_post(
    s: SDW,
    client: RequestClient,
    mut to_create: web::Json<VerificationTargetWrapper<PostToCreate>>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    check_user(&s, tx.as_mut(), user, WhatToDo::WritePost).await?;
    to_create.verify()?;

    check_category(
        &s,
        tx.as_mut(),
        to_create.target.category_id,
        user,
        WhatToDo::WritePost,
    )
    .await?;

    {
        let tar = &mut to_create.target;
        let max = s.core.get_config().top_index_max as usizedb;
        if tar.top_index > 0 {
            if !s
                .category
                .can_manage(tx.as_mut(), tar.category_id, user)
                .await?
            {
                tar.top_index = 0;
            } else if !user.unwrap().is_admin() && tar.top_index > max {
                tar.top_index = max;
            }
        }
    }
    tx.commit().await.unwrap();

    if s.core.get_config().auto_fetch_post_cover {
        let cover_url = to_create
            .target
            .content_type
            .get_first_image_url(&to_create.target.content);
        to_create.target.cover_url = cover_url
    }

    let post = s
        .post
        .create_post(user.unwrap().id, to_create.into_inner().target)
        .await?;
    Ok(HttpResponse::Ok().json(post))
}

#[put("/post/{id}")]
pub async fn update_post(
    s: SDW,
    client: RequestClient,
    mut to_update: web::Json<VerificationTargetWrapper<PostToUpdate>>,
    path: web::Path<(usizedb,)>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    check_user(&s, tx.as_mut(), client.get_user(), WhatToDo::WritePost).await?;
    to_update.verify()?;
    let (pid,) = path.into_inner();
    let post = check_post(&s, tx.as_mut(), pid, user, WhatToDo::WritePost).await?;

    {
        let tar = &mut to_update.target;
        let max = s.core.get_config().top_index_max as usizedb;
        if tar.top_index > 0 {
            if !s
                .category
                .can_manage(tx.as_mut(), post.category_id, user)
                .await?
            {
                tar.top_index = 0;
            } else if !user.unwrap().is_admin() && tar.top_index > max {
                tar.top_index = max;
            }
        }
    }
    tx.commit_unwrap().await;

    if post.cover_url != to_update.target.cover_url {
        let mut tx = s.core.begin_unwrap(true).await;
        // change old cover permanent to false to clear resource.
        if let Some(old_cover) = &post.cover_url {
            if let Some(key) = s.storage.try_parse_url_to_key(&old_cover) {
                s.storage
                    .remark(
                        tx.as_mut(),
                        key,
                        ObjectFlag::PostCover,
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
                        ObjectFlag::PostCover,
                        client.get_user_unwrap().id,
                        true,
                    )
                    .await?;
            }
        }
        tx.commit_unwrap().await;
    }
    if to_update.target.cover_url.is_none() && s.core.get_config().auto_fetch_post_cover {
        let cover_url = to_update
            .target
            .content_type
            .get_first_image_url(&to_update.target.content);
        to_update.target.cover_url = cover_url
    }

    let mut tx = s.core.begin_unwrap(true).await;
    match s
        .post
        .update_post(
            tx.as_mut(),
            pid,
            user.unwrap().id,
            to_update.into_inner().target,
        )
        .await?
    {
        Some(p) => {
            tx.commit().await.unwrap();
            Ok(HttpResponse::Ok().json(p))
        }
        None => ApiError::no_post_found().to_err(),
    }
}

#[get("/posts")]
pub async fn get_postlinks_with_algorithm(
    s: SDW,
    query: web::Query<GetPostsQuery>,
    client: RequestClient,
) -> Result<web::Json<GetDatasExtended<PostInfo>>, ApiError> {
    let index = query.index;
    let limit = query.limit;
    if limit as usize > s.core.get_config().fetch_limit {
        return ApiError::fetch_limit().to_err();
    }
    let extended = query.extended;
    let time = PostFilterTime::from_str(&query.time, query.time_num);
    let query = query.into_inner();
    let mut tx = s.core.begin_unwrap(false).await;
    let items = s
        .post
        .get_postlinks_with_algorithm(
            tx.as_mut(),
            index,
            limit,
            query.sort,
            time,
            query.category_id,
            query.created_by_id,
            query.distinct,
            query.top_order_enable,
        )
        .await?;
    let total = s
        .post
        .get_all_post_count(
            tx.as_mut(),
            time,
            query.category_id,
            query.created_by_id,
            query.distinct,
            query.top_order_enable,
        )
        .await?;

    let mut builder = GetDatasExtendedBuilder::new(&s);
    if extended {
        for p in &items {
            builder
                .extend_categories(tx.as_mut(), vec![p.category_id])
                .await?;
            builder
                .extend_users(
                    tx.as_mut(),
                    vec![p.created_by_id, p.last_edit_by_id, p.last_comment_by_id],
                )
                .await?;
            let post_ids = items.iter().map(|p| p.id.clone()).collect();
            if let Some(user) = client.get_user() {
                builder
                    .extend_posts_like_status(tx.as_mut(), post_ids, user.id)
                    .await?;
            }
        }
    }
    let data = builder.set_data(ListSlice { items, total }).build();
    Ok(web::Json(data))
}

#[get("/post/{id}")]
pub async fn get_post(
    s: SDW,
    client: RequestClient,
    query: web::Query<GetPostQuery>,
    path: web::Path<(usizedb,)>,
) -> Result<HttpResponse, ApiError> {
    let (pid,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(false).await;
    check_post(&s, tx.as_mut(), pid, client.get_user(), WhatToDo::None).await?;
    let mut post = s.post.get_post(tx.as_mut(), pid, query.full).await?.unwrap();
    tx.commit().await.unwrap();
    if post.content.is_some() {
        s.post.increment_views(pid).await?;
        post.views += 1;
    }
    Ok(HttpResponse::Ok().json(post))
}

#[put("/post_status/{id}")]
pub async fn set_post_status(
    s: SDW,
    client: RequestClient,
    path: web::Path<(usizedb,)>,
    body: web::Json<SetStatusBody>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user();
    let (id,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(true).await;
    can_manage_post(&s, tx.as_mut(), id, user).await?;
    if s.post
        .set_status(tx.as_mut(), id, body.into_inner().status)
        .await?
    {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_post_found().to_err()
    }
}
