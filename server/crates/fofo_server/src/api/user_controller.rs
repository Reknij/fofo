mod model;

use actix_web::{get, http::StatusCode, post, put, web, HttpResponse};
use fofo_utils::usizedb;
use storage::object_marker::model::ObjectFlag;
use user_system::model::{SafeUserInfo, UserStatus, UserToCreate, UserToUpdate, UserType};

use crate::{
    api::{
        api_error::{ApiError, DetailErrorCode},
        user_controller::model::{
            AuthAndUser, GetUsersQuery, SetStatusBody, ToDeleteUsers, ToLoginUser, UserAuthQuery
        },
        util::{
            check_user, check_verification_and_pass_it, GetDatasExtended, GetDatasExtendedBuilder, ListSlice, VerificationTargetWrapper, WhatToDo
        },
        SDW,
    },
    request_client::RequestClient,
};

#[post("/user")]
pub async fn create_user(
    s: SDW,
    client: RequestClient,
    to_create: web::Json<VerificationTargetWrapper<UserToCreate>>,
) -> Result<HttpResponse, ApiError> {
    if client.is_logined() {
        return ApiError::new(
            StatusCode::METHOD_NOT_ALLOWED,
            DetailErrorCode::NoLoginRequired,
            "Login already can't create account.",
        )
        .to_err();
    }
    to_create.verify()?;
    let config = s.core.get_config();
    if !config.open_register && !client.get_user_unwrap().is_admin() {
        return ApiError::unsupported_api().to_err();
    }
    let mut tx = s.core.begin_unwrap(true).await;
    check_verification_and_pass_it(&s, tx.as_mut(), to_create.verification.as_ref()).await?;

    if s.user
        .get_user_by_email(tx.as_mut(), &to_create.target.email)
        .await?
        .is_some()
    {
        return ApiError::unique_email_required().to_err();
    }

    if s.user
        .get_user_by_username(tx.as_mut(), &to_create.target.username)
        .await?
        .is_some()
    {
        return ApiError::unique_username_required().to_err();
    }
    tx.commit_unwrap().await;
    let user = s
        .user
        .create_user(
            to_create.into_inner().target,
            UserStatus::Active,
            UserType::General,
        )
        .await?;
    let mut tx = s.core.begin_unwrap(true).await;
    let auth = s.user.get_and_save_auth(tx.as_mut(), &user).await?;
    tx.commit_unwrap().await;
    let anu = AuthAndUser { auth, user };
    Ok(HttpResponse::Ok().json(anu))
}

#[post("/delete_user/{id}")]
pub async fn delete_user_by_id(
    s: SDW,
    client: RequestClient,
    path: web::Path<(usizedb,)>,
    to_delete: web::Json<VerificationTargetWrapper<()>>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }
    let (id,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(true).await;
    check_verification_and_pass_it(&s, tx.as_mut(), to_delete.verification.as_ref()).await?;

    if !s.user.get_user(tx.as_mut(), id).await?.is_some() {
        return ApiError::no_user_found().to_err();
    }

    let current = client.get_user_unwrap();
    if !current.is_admin() && id != current.id {
        return ApiError::no_permission("You not permission to delete user.").to_err();
    }
    let r = s.user.delete_user(tx.as_mut(), id).await?;
    tx.commit().await.unwrap();
    Ok(HttpResponse::Ok().json(r))
}

#[post("/delete_users")]
pub async fn delete_users_by_ids(
    s: SDW,
    client: RequestClient,
    to_delete: web::Json<ToDeleteUsers>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }

    if !client.get_user_unwrap().is_admin() {
        return ApiError::only_admin().to_err();
    }

    let mut tx = s.core.begin_unwrap(false).await;
    let mut deleted_num = 0;
    for id in to_delete.ids.clone() {
        if s.user.delete_user(tx.as_mut(), id).await? {
            deleted_num += 1;
        }
    }
    tx.commit().await.unwrap();
    Ok(HttpResponse::Ok().json(deleted_num))
}

#[put("/user/{id}")]
pub async fn update_user(
    s: SDW,
    client: RequestClient,
    to_update: web::Json<VerificationTargetWrapper<UserToUpdate>>,
    path: web::Path<(usizedb,)>,
) -> Result<HttpResponse, ApiError> {
    if !client.is_logined() {
        return ApiError::login_required().to_err();
    }

    to_update.verify()?;

    let mut tx = s.core.begin_unwrap(true).await;
    check_verification_and_pass_it(&s, tx.as_mut(), to_update.verification.as_ref()).await?;
    check_user(&s, tx.as_mut(), client.get_user(), WhatToDo::None).await?;
    let (uid,) = path.into_inner();
    match s.user.get_user(tx.as_mut(), uid).await? {
        Some(user) => {
            if user.email != to_update.target.email
                && s.user
                    .get_user_by_email(tx.as_mut(), &to_update.target.email)
                    .await?
                    .is_some()
            {
                return ApiError::unique_email_required().to_err();
            }
        }
        None => return ApiError::no_user_found().to_err(),
    }
    tx.commit_unwrap().await;

    let current = client.get_user_unwrap();
    if current.id == uid || current.is_admin() {
        if to_update.target.avatar_url != current.avatar_url {
            let mut tx = s.core.begin_unwrap(true).await;
            // change current avatar permanent to false to clear resource.
            if let Some(old_avatar) = &current.avatar_url {
                if let Some(key) = s.storage.try_parse_url_to_key(&old_avatar) {
                    s.storage
                        .remark(tx.as_mut(), key, ObjectFlag::UserAvatar, uid, false)
                        .await?;
                }
            }
            if let Some(avatar_url) = &to_update.target.avatar_url {
                // change newest avatar permanent to true to keep it permanent.
                if let Some(key) = s.storage.try_parse_url_to_key(&avatar_url) {
                    s.storage
                        .remark(tx.as_mut(), key, ObjectFlag::UserAvatar, uid, true)
                        .await?;
                }
            }
            tx.commit_unwrap().await;
        }
        let mut tx = s.core.begin_unwrap(true).await;
        match s
            .user
            .update_user(tx.as_mut(), uid, to_update.into_inner().target)
            .await?
        {
            Some(u) => {
                tx.commit_unwrap().await;
                Ok(HttpResponse::Ok().json(u))
            }
            None => ApiError::no_user_found().to_err(),
        }
    } else {
        ApiError::no_permission("You not permission to update user.").to_err()
    }
}

#[put("/user_status/{id}")]
pub async fn set_user_status(
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
    if s.user
        .set_status(tx.as_mut(), id, body.into_inner().status)
        .await?
    {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::no_user_found().to_err()
    }
}

#[get("/user/{id}")]
pub async fn get_user_by_id(
    s: SDW,
    client: RequestClient,
    path: web::Path<(usizedb,)>,
) -> Result<HttpResponse, ApiError> {
    let (user_id,) = path.into_inner();
    let mut tx = s.core.begin_unwrap(false).await;
    if client.is_logined() && client.get_user_unwrap().is_admin() {
        match s.user.get_user(tx.as_mut(), user_id).await? {
            Some(u) => Ok(HttpResponse::Ok().json(u)),
            None => ApiError::no_user_found().to_err(),
        }
    } else {
        match s.user.get_safe_user(tx.as_mut(), user_id).await? {
            Some(u) => Ok(HttpResponse::Ok().json(u)),
            None => ApiError::no_user_found().to_err(),
        }
    }
}

#[get("/users")]
pub async fn get_users(
    s: SDW,
    query: web::Query<GetUsersQuery>,
) -> Result<web::Json<GetDatasExtended<SafeUserInfo>>, ApiError> {
    let index = query.index * query.limit;
    let limit = query.limit;
    let sort = if query.desc {
        format!("{} DESC", query.sort.to_string())
    } else {
        query.sort.to_string()
    };
    let f = format!("ORDER BY {} LIMIT {} OFFSET {}", sort, limit, index);
    let mut tx = s.core.begin_unwrap(false).await;
    let items = s.user.get_safe_users(tx.as_mut(), &f).await?;
    let total = s.user.get_count(index, limit).await?;

    let mut builder = GetDatasExtendedBuilder::new(&s);
    if query.extended {
        for u in &items {
            builder
                .extend_groups(tx.as_mut(), u.group_ids.to_owned())
                .await?;
        }
    }
    let data = builder.set_data(ListSlice { items, total }).build();
    Ok(web::Json(data))
}

#[post("/login_user")]
pub async fn login_user(
    s: SDW,
    to_login: web::Json<VerificationTargetWrapper<ToLoginUser>>,
) -> Result<HttpResponse, ApiError> {
    to_login.verify()?;
    let mut tx = s.core.begin_unwrap(true).await;
    check_verification_and_pass_it(&s, tx.as_mut(), to_login.verification.as_ref()).await?;

    let user = match s
        .user
        .get_user_by_username(tx.as_mut(), &to_login.target.username)
        .await?
    {
        Some(user) => user,
        None => return ApiError::no_user_found().to_err(),
    };

    if &user.password != &to_login.target.password {
        return ApiError::password_not_match().to_err();
    }

    let auth = s.user.get_and_save_auth(tx.as_mut(), &user).await?;
    let anu = AuthAndUser { auth, user };
    tx.commit().await.unwrap();
    Ok(HttpResponse::Ok().json(anu))
}

#[get("/logout_user")]
pub async fn logout_user(
    s: SDW,
    query: web::Query<UserAuthQuery>,
) -> Result<HttpResponse, ApiError> {
    let mut tx = s.core.begin_unwrap(true).await;
    if s.user.remove_auth(tx.as_mut(), &query.auth).await? {
        tx.commit().await.unwrap();
        Ok(HttpResponse::Ok().finish())
    } else {
        ApiError::authorization_error().to_err()
    }
}

#[get("/revert_user")]
pub async fn revert_user(
    s: SDW,
    query: web::Query<UserAuthQuery>,
) -> Result<HttpResponse, ApiError> {
    match s.user.revert(&query.auth).await? {
        Some(u) => Ok(HttpResponse::Ok().json(u)),
        None => ApiError::authorization_error().to_err(),
    }
}
