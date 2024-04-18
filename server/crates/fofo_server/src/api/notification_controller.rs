mod model;

use actix_web::{get, web, HttpResponse};

use crate::{
    api::{
        notification_controller::model::SetUserNotificationReadedQuery,
        util::{GetDatasExtended, GetDatasExtendedBuilder, ListSlice},
        SDW,
    },
    request_client::RequestClient,
};
use notification_system::model::{UserNotification, UserNotificationType};

use self::model::GetUserNotificationsQuery;

use super::api_error::ApiError;

#[get("/user_notifications")]
pub async fn get_user_notifications(
    s: SDW,
    query: web::Query<GetUserNotificationsQuery>,
    client: RequestClient,
) -> Result<web::Json<GetDatasExtended<UserNotification>>, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let index = query.index;
    let limit = query.limit;
    if limit as usize > s.core.get_config().fetch_limit {
        return ApiError::fetch_limit().to_err();
    }
    let user = client.get_user_unwrap();
    let mut tx = s.core.begin_unwrap(false).await;
    let items = s
        .notification
        .get_user_notifications(tx.as_mut(), user.id, index, limit, query.only_unread)
        .await?;
    let total = s
        .notification
        .get_user_notification_count(tx.as_mut(), user.id, query.only_unread)
        .await?;

    let mut builder = GetDatasExtendedBuilder::new(&s);
    if query.extended {
        for un in &items {
            builder
                .extend_users(tx.as_mut(), vec![un.created_by_id])
                .await?;
            let ids = vec![un.ref_id];
            match un.n_type {
                UserNotificationType::Comment
                | UserNotificationType::ReplyComment
                | UserNotificationType::LikeComment
                | UserNotificationType::DislikeComment => {
                    builder.extend_comments(tx.as_mut(), ids).await?
                }
                UserNotificationType::LikePost | UserNotificationType::DislikePost => {
                    builder.extend_posts(tx.as_mut(), ids).await?
                }
            };
        }
    }
    tx.commit().await?;
    let data = builder.set_data(ListSlice { items, total }).build();
    Ok(web::Json(data))
}

#[get("/set_user_notification_readed")]
pub async fn set_user_notification_readed(
    s: SDW,
    query: web::Query<SetUserNotificationReadedQuery>,
    client: RequestClient,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let user = client.get_user_unwrap();
    s.notification
        .set_user_notification_read_status(user.id, query.id, query.readed)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
