mod model;

use actix_web::{get, put, web, HttpResponse};

use crate::{
    api::{like_controller::model::GetLikeStatusQuery, SDW},
    request_client::RequestClient,
    ServerData,
};
use fofo_utils::usizedb;
use like_system::model::LikeStatusFlag;
use notification_system::model::{UserNotificationArguments, UserNotificationType};

use self::model::{LikeAction, LikeActionBody};

use super::{
    api_error::ApiError,
    util::{check_comment, check_post, check_user, WhatToDo},
};

#[get("/like_status")]
pub async fn get_like_status(
    s: SDW,
    q: web::Query<GetLikeStatusQuery>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.core.begin_unwrap(false).await;
    let s = s
        .like
        .get_like_status(tx.as_mut(), q.flag_ref_id, q.flag, q.created_by_id)
        .await?;
    Ok(HttpResponse::Ok().json(s))
}

#[put("/like_action/{id}")]
pub async fn like_action(
    s: SDW,
    path: web::Path<(usizedb,)>,
    body: web::Json<LikeActionBody>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    match body.action {
        LikeAction::Like => like_or_dislike(&s, path, client, Some(true), body.flag).await,
        LikeAction::Dislike => like_or_dislike(&s, path, client, Some(false), body.flag).await,
        LikeAction::UnknownLike => like_or_dislike(&s, path, client, None, body.flag).await,
    }
}

pub async fn like_or_dislike(
    s: &ServerData,
    path: web::Path<(usizedb,)>,
    client: RequestClient,
    is_like: Option<bool>,
    flag: LikeStatusFlag,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }

    let mut tx = s.core.begin_unwrap(false).await;
    let w = if flag == LikeStatusFlag::TargetPost {
        WhatToDo::LikePost
    } else {
        WhatToDo::LikeComment
    };
    let user = client.get_user();
    check_user(&s, tx.as_mut(), user, w).await?; // like also is comment.
    let (id,) = path.into_inner();
    let target_user_id = match flag {
        LikeStatusFlag::TargetPost => {
            let post = check_post(&s, tx.as_mut(), id, user, w).await?;
            post.created_by_id
        }
        LikeStatusFlag::TargetComment => {
            let (_, comment) = check_comment(&s, tx.as_mut(), id, user, w).await?;
            comment.created_by_id
        }
    };
    tx.commit_unwrap().await;
    let status = s
        .like
        .set_like_status(id, flag, user.unwrap().id, is_like)
        .await?;
    if let Some(is_like) = is_like {
        let n_type = match flag {
            LikeStatusFlag::TargetPost => {
                if is_like {
                    UserNotificationType::LikePost
                } else {
                    UserNotificationType::DislikePost
                }
            }
            LikeStatusFlag::TargetComment => {
                if is_like {
                    UserNotificationType::LikeComment
                } else {
                    UserNotificationType::DislikeComment
                }
            }
        };
        let s = s.clone();
        tokio::spawn(async move {
            s.notification
                .create_user_notification(
                    client.get_user_unwrap().id,
                    UserNotificationArguments {
                        ref_id: id,
                        target_user_id,
                        n_type,
                    },
                )
                .await
                .unwrap();
        });
    }
    Ok(HttpResponse::Ok().json(status))
}
