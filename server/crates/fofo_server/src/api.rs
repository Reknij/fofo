use actix_web::{web, Scope};

use super::ServerData;

pub mod api_error;
mod category_controller;
mod comment_controller;
mod group_controller;
mod like_controller;
mod notification_controller;
mod post_controller;
mod server_controller;
mod storage_controller;
mod user_controller;
mod util;
mod verification_controller;

pub(crate) type SDW = web::Data<ServerData>;

pub fn get_api_services() -> Scope {
    let scope = web::scope("/api")
        // verification controller
        .service(verification_controller::get_verification)
        // user controller
        .service(user_controller::create_user)
        .service(user_controller::delete_user_by_id)
        .service(user_controller::delete_users_by_ids)
        .service(user_controller::get_user_by_id)
        .service(user_controller::get_users)
        .service(user_controller::update_user)
        .service(user_controller::login_user)
        .service(user_controller::logout_user)
        .service(user_controller::revert_user)
        .service(user_controller::set_user_status)
        // group controller
        .service(group_controller::create_group)
        .service(group_controller::delete_group)
        .service(group_controller::update_group)
        .service(group_controller::get_group)
        .service(group_controller::get_groups)
        .service(group_controller::set_group_status)
        // post controller
        .service(post_controller::create_post)
        .service(post_controller::get_postlinks_with_algorithm)
        .service(post_controller::get_post)
        .service(post_controller::update_post)
        .service(post_controller::set_post_status)
        // comment controller
        .service(comment_controller::create_comment)
        .service(comment_controller::get_comment)
        .service(comment_controller::get_comments)
        .service(comment_controller::update_comment)
        .service(comment_controller::set_comment_status)
        // like controller
        .service(like_controller::get_like_status)
        .service(like_controller::like_action)
        // category controller
        .service(category_controller::create_category)
        .service(category_controller::delete_category)
        .service(category_controller::update_category)
        .service(category_controller::get_categories)
        .service(category_controller::get_category)
        .service(category_controller::set_category_status)
        // storage service controller
        .service(storage_controller::presign_put_url)
        .service(storage_controller::upload_presigned)
        .service(storage_controller::get_static_file)
        // notification controller
        .service(notification_controller::get_user_notifications)
        .service(notification_controller::set_user_notification_readed)
        // server controller
        .service(server_controller::get_server_info);
    scope
}
