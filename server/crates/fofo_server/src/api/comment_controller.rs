use crate::{
    api::{
        comment_controller::model::{GetCommentsQuery, SetStatusBody},
        util::{
            can_manage_comment, check_comment, check_post, check_user, GetDatasExtended,
            GetDatasExtendedBuilder, ListSlice, VerificationTargetWrapper, WhatToDo,
        },
        SDW,
    },
    request_client::RequestClient,
};
use actix_web::{get, post, put, web, HttpResponse};
use comment_system::model::{CommentInfo, CommentToCreate, CommentToUpdate};
use fofo_utils::usizedb;
use notification_system::model::{UserNotificationArguments, UserNotificationType};

use super::api_error::ApiError;

mod model;

#[get("/comment/{id}")]
pub async fn get_comment(
    s: SDW,
    path: web::Path<(usizedb,)>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    let (cid,) = path.into_inner();
    check_comment(&s, tx.as_mut(), cid, user, WhatToDo::None).await?;
    let v = s.comment.get_comment(tx.as_mut(), cid).await?.unwrap();
    Ok(HttpResponse::Ok().json(v))
}

#[get("/comments")]
pub async fn get_comments(
    s: SDW,
    query: web::Query<GetCommentsQuery>,
    client: RequestClient,
) -> Result<web::Json<GetDatasExtended<CommentInfo>>, ApiError> {
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    let post_id = query.post_id;
    let parent_id = query.parent_id;
    let index = query.index;
    let limit = query.limit;

    if limit as usize > s.core.get_config().fetch_limit {
        return ApiError::fetch_limit().to_err();
    }

    let desc = query.desc;
    let extended = query.extended;
    let top_order_enable = query.top_order_enable;
    let sort = query.into_inner().sort;

    let post = check_post(&s, tx.as_mut(), post_id, user, WhatToDo::None).await?;
    let items = s
        .comment
        .get_comments(
            tx.as_mut(),
            post_id,
            parent_id,
            sort,
            index,
            limit,
            desc,
            top_order_enable,
        )
        .await?;
    let total = post.total_comment_post;

    let mut builder = GetDatasExtendedBuilder::new(&s);
    if extended {
        for c in &items {
            builder
                .extend_users(
                    tx.as_mut(),
                    vec![c.created_by_id, c.last_edit_by_id, c.reply_user_id],
                )
                .await?;
            let comment_ids = items.iter().map(|c| c.id.clone()).collect();
            if let Some(user) = user {
                builder
                    .extend_comments_like_status(tx.as_mut(), comment_ids, user.id)
                    .await?;
            }
        }
    }
    let data = builder.set_data(ListSlice { items, total }).build();
    Ok(web::Json(data))
}

#[post("/comment")]
pub async fn create_comment(
    s: SDW,
    client: RequestClient,
    mut to_create: web::Json<VerificationTargetWrapper<CommentToCreate>>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    check_user(&s, tx.as_mut(), user, WhatToDo::WriteComment).await?;
    let post = check_post(
        &s,
        tx.as_mut(),
        to_create.target.post_id,
        user,
        WhatToDo::WriteComment,
    )
    .await?;

    {
        let tar = &mut to_create.target;
        tar.category_id = post.category_id;
        let max = s.core.get_config().top_index_max as usizedb;
        if tar.top_index > 0 {
            if post.created_by_id != user.unwrap().id
                && !s
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

    let n_type;
    if to_create.target.reply_comment_id > 0 {
        let (_, reply_comment) = check_comment(
            &s,
            tx.as_mut(),
            to_create.target.reply_comment_id,
            user,
            WhatToDo::ReplyComment,
        )
        .await?;

        if reply_comment.post_id != to_create.target.post_id {
            return ApiError::same_post_comment_required().to_err();
        }
        to_create.target.parent_id = if reply_comment.reply_comment_id > 0 {
            reply_comment.parent_id
        } else {
            // The comment currently being replied to is a direct reply to the post
            reply_comment.id
        };
        to_create.target.reply_user_id = reply_comment.created_by_id;
        n_type = UserNotificationType::ReplyComment;
    } else if to_create.target.parent_id > 0 {
        return ApiError::reply_comment_required().to_err();
    } else {
        to_create.target.reply_user_id = post.created_by_id;
        n_type = UserNotificationType::Comment;
    }
    tx.commit().await.unwrap();

    let current = user.unwrap();
    let target_user_id = to_create.target.reply_user_id;
    let comment = s
        .comment
        .create_comment(current.id, to_create.into_inner().target)
        .await?;
    let current_id = current.id;
    let comment_id = comment.id;
    tokio::spawn(async move {
        s.notification
            .create_user_notification(
                current_id,
                UserNotificationArguments {
                    ref_id: comment_id,
                    target_user_id,
                    n_type,
                },
            )
            .await
            .expect("Create notification failed.")
    });
    Ok(HttpResponse::Ok().json(comment))
}

#[put("/comment_status/{id}")]
pub async fn set_comment_status(
    s: SDW,
    client: RequestClient,
    path: web::Path<(usizedb,)>,
    body: web::Json<SetStatusBody>,
) -> Result<HttpResponse, ApiError> {
    let (id,) = path.into_inner();
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(true).await;

    can_manage_comment(&s, tx.as_mut(), id, user).await?;

    s.comment
        .set_status(tx.as_mut(), id, body.into_inner().status)
        .await?;
    tx.commit().await.unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[put("/comment/{id}")]
pub async fn update_comment(
    s: SDW,
    client: RequestClient,
    mut to_update: web::Json<VerificationTargetWrapper<CommentToUpdate>>,
    path: web::Path<(usizedb,)>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user();
    let mut tx = s.core.begin_unwrap(false).await;
    check_user(&s, tx.as_mut(), user, WhatToDo::WriteComment).await?;
    let (id,) = path.into_inner();
    let (post, _) = check_comment(&s, tx.as_mut(), id, user, WhatToDo::WriteComment).await?;

    {
        let tar = &mut to_update.target;
        let max = s.core.get_config().top_index_max as usizedb;
        if tar.top_index > 0 {
            if post.created_by_id != user.unwrap().id
                && !s
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
    let mut tx = s.core.begin_unwrap(true).await;
    let current = user.unwrap();
    match s
        .comment
        .update_comment(tx.as_mut(), id, current.id, to_update.into_inner().target)
        .await?
    {
        Some(p) => {
            tx.commit().await.unwrap();
            Ok(HttpResponse::Ok().json(p))
        }
        None => ApiError::no_post_found().to_err(),
    }
}
