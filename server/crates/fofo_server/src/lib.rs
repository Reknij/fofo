use actix_cors::Cors;
use actix_governor::{
    governor::clock::{Clock, DefaultClock},
    Governor, GovernorConfigBuilder, KeyExtractor,
};
use actix_web::{
    dev::ServiceRequest,
    http::{header, Method},
    web::{self, PayloadConfig},
    App, HttpServer,
};
use anyhow::Result;
use api::{api_error::ApiError, SDW};
use category_system::CategorySystem;
use comment_system::CommentSystem;
use group_system::GroupSystem;
use like_system::LikeSystem;
use notification_system::NotificationSystem;
use post_system::PostSystem;
use request_client::get_auth_from_request;
use shared_core::SharedCore;
use storage::S3Ref;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use user_system::UserSystem;
use vertification_system::VerificationSystem;

mod api;
mod request_client;

#[derive(Debug, Clone)]
pub struct ServerData {
    pub core: SharedCore,
    pub storage: S3Ref,
    pub user: UserSystem,
    pub group: GroupSystem,
    pub post: PostSystem,
    pub comment: CommentSystem,
    pub like: LikeSystem,
    pub category: CategorySystem,
    pub verification: VerificationSystem,
    pub notification: NotificationSystem,
}

#[derive(Clone)]
struct CustomKeyExtractor;
impl CustomKeyExtractor {
    fn get_ip_str_from_request(&self, req: &ServiceRequest) -> Result<String, ApiError> {
        let config = req.app_data::<SDW>().unwrap().core.get_config();
        let ip = if config.forwarded_ip {
            match req.headers().get(header::X_FORWARDED_FOR) {
                Some(v) => match v.to_str() {
                    Ok(vstr) => vstr.to_owned(),
                    Err(_) => {
                        return ApiError::internal_error("Can't parse the header value to str.")
                            .to_err()
                    }
                },
                None => req
                    .peer_addr()
                    .map(|socket| socket.ip())
                    .ok_or_else(|| ApiError::no_ip_address_found())?
                    .to_string(),
            }
        } else {
            req.peer_addr()
                .map(|socket| socket.ip())
                .ok_or_else(|| ApiError::no_ip_address_found())?
                .to_string()
        };
        Ok(ip)
    }
}

impl KeyExtractor for CustomKeyExtractor {
    type Key = String;

    type KeyExtractionError = ApiError;

    fn extract(
        &self,
        req: &ServiceRequest,
    ) -> std::result::Result<Self::Key, Self::KeyExtractionError> {
        let conf = req.app_data::<SDW>().unwrap().core.get_config();
        if let (Some(bypass_key), Some(v)) = (&conf.bypass_key, req.headers().get("x-bypass-key")) {
            match v.to_str() {
                Ok(vkey) => {
                    if vkey == bypass_key {
                        return Ok("bypass".to_owned());
                    }
                }
                Err(_) => error!("Can't parse the header value to str."),
            }
        }
        let req_path = req.path();
        match req.path() {
            "/api/login_user"
            | "/api/create_user"
            | "/api/update_user"
            | "/api/verification"
            | "/api/presign_put_url" => {
                let ip = self.get_ip_str_from_request(req)?;
                Ok(format!("{ip}-{req_path}"))
            }
            _ => match req.method() {
                &Method::PUT | &Method::POST => match get_auth_from_request(req.request()) {
                    Some(authorization) => Ok(format!("{authorization}-{req_path}")),
                    None => {
                        let ip = self.get_ip_str_from_request(req)?;
                        Ok(format!("{ip}-{req_path}"))
                    }
                },
                _ => Ok("bypass".to_owned()),
            },
        }
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &actix_governor::governor::NotUntil<
            actix_governor::governor::clock::QuantaInstant,
        >,
        mut response: actix_web::HttpResponseBuilder,
    ) -> actix_web::HttpResponse {
        let wait_time = negative
            .wait_time_from(DefaultClock::default().now())
            .as_secs();
        let time_str = format!("{wait_time}(s)");
        response.json(ApiError::too_many_requests(&time_str).detail)
    }

    fn whitelisted_keys(&self) -> Vec<Self::Key> {
        vec!["bypass".to_owned()]
    }
}

pub async fn run(core: SharedCore, address: &str, port: u16) -> Result<()> {
    let config = core.get_config();
    
    let storage = S3Ref::new(core.clone()).await;
    let user = UserSystem::new(core.clone(), storage.clone()).await;
    let group = GroupSystem::new(core.clone()).await;
    let category = CategorySystem::new(core.clone(), storage.clone()).await;
    let post = PostSystem::new(core.clone(), storage.clone()).await;
    let comment = CommentSystem::new(core.clone()).await;
    let like = LikeSystem::new(core.clone()).await;
    let verification = VerificationSystem::new(core.clone(), storage.clone()).await;
    let notification = NotificationSystem::new(core.clone()).await;

    let server_data = ServerData {
        core,
        storage,
        user,
        group,
        post,
        comment,
        like,
        category,
        verification,
        notification,
    };

    info!("Running server in http://{}:{}", address, port);

    #[cfg(debug_assertions)]
    debug_data_initial(server_data.clone()).await;

    let s = web::Data::new(server_data);

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(5)
        .burst_size(15)
        .key_extractor(CustomKeyExtractor)
        .finish()
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(TracingLogger::default())
            .wrap(Cors::permissive())
            .app_data(s.clone())
            .app_data(PayloadConfig::new(config.local.max_bytes))
            .service(api::get_api_services())
    })
    .bind((address, port))?
    .run()
    .await?;

    Ok(())
}

#[cfg(debug_assertions)]
async fn debug_data_initial(s: ServerData) {
    let log = false;
    let mod1 = s
        .user
        .create_user(
            user_system::model::UserToCreate {
                alias: "Moderator 1".to_owned(),
                username: "mod01".to_owned(),
                password: "moderator1".to_owned(),
                email: "mod1@adm.com".to_owned(),
            },
            user_system::model::UserStatus::Active,
            user_system::model::UserType::General,
        )
        .await
        .unwrap();

    let mod2 = s
        .user
        .create_user(
            user_system::model::UserToCreate {
                alias: "Moderator 2".to_owned(),
                username: "mod02".to_owned(),
                password: "moderator2".to_owned(),
                email: "mod2@adm.com".to_owned(),
            },
            user_system::model::UserStatus::Active,
            user_system::model::UserType::General,
        )
        .await
        .unwrap();

    let user1 = s
        .user
        .create_user(
            user_system::model::UserToCreate {
                alias: "User1".to_owned(),
                username: "user1".to_owned(),
                password: "user1234".to_owned(),
                email: "user1@adm.com".to_owned(),
            },
            user_system::model::UserStatus::Active,
            user_system::model::UserType::General,
        )
        .await
        .unwrap();

    let general_category = s
        .category
        .create_category(category_system::model::CategoryToCreate {
            title: "General Category".to_owned(),
            description: "# For General Category\nI am category for test.".to_owned(),
            description_content_type: fofo_utils::ContentType::Markdown,
            status: category_system::model::CategoryStatus::Active,
            read_level: user_system::model::UserType::General,
            write_level: user_system::model::UserType::General,
            comment_level: user_system::model::UserType::General,
            moderator_ids: vec![mod1.id, mod2.id],
            group_ids: vec![],
            cover_url: None,
        })
        .await
        .unwrap();

    let guest_category = s
        .category
        .create_category(category_system::model::CategoryToCreate {
            title: "Guest Category".to_owned(),
            description: "# For Guest Category\nI am category for test.".to_owned(),
            description_content_type: fofo_utils::ContentType::Markdown,
            status: category_system::model::CategoryStatus::Active,
            read_level: user_system::model::UserType::Guest,
            write_level: user_system::model::UserType::General,
            comment_level: user_system::model::UserType::General,
            moderator_ids: vec![mod1.id, mod2.id],
            group_ids: vec![],
            cover_url: None,
        })
        .await
        .unwrap();

    let mut tasks1: Vec<tokio::task::JoinHandle<()>> = (0..9999)
        .map(|number| -> tokio::task::JoinHandle<()> {
            let s = s.clone();

            let mod1 = mod1.clone();
            let _mod2 = mod2.clone();
            tokio::spawn(async move {
                let p = s
                    .post
                    .create_post(
                        mod1.id,
                        post_system::model::PostToCreate {
                            title: format!("Mod user's Post number {}", number),
                            content: format!("# hi num {}\nI am mod1\n# hi again\nhahaha.", number),
                            content_type: fofo_utils::ContentType::Markdown,
                            category_id: general_category.id,
                            tags: vec![
                                "tag1".to_owned(),
                                "tag2".to_owned(),
                                "tag3".to_owned(),
                                "tag4".to_owned(),
                                "tag5".to_owned(),
                            ],
                            cover_url: None,
                            top_index: 0,
                        },
                    )
                    .await
                    .unwrap();
                if log {
                    info!("created post: {}", &p.title);
                }
            })
        })
        .collect();
    let mut tasks2: Vec<_> = (0..100)
        .map(|number| {
            let s = s.clone();

            let user1 = user1.clone();
            let mod1 = mod1.clone();
            let mod2 = mod2.clone();
            tokio::spawn(async move {
                let p = s
                    .post
                    .create_post(
                        user1.id,
                        post_system::model::PostToCreate {
                            title: format!("General user's Post number {}", number),
                            content: format!(
                                "# hi num {}\nI am user1\n# hi again\nhahaha.",
                                number
                            ),
                            content_type: fofo_utils::ContentType::Markdown,
                            category_id: guest_category.id,
                            tags: vec![
                                "tag1".to_owned(),
                                "tag2".to_owned(),
                                "tag3".to_owned(),
                                "tag4".to_owned(),
                                "tag5".to_owned(),
                            ],
                            cover_url: None,
                            top_index: 0,
                        },
                    )
                    .await
                    .unwrap();
                if log {
                    info!("created post: {}", &p.title);
                }
                let comments_task: Vec<_> = (0..34)
                    .map(|number| {
                        let p = p.clone();
                        let s = s.clone();

                        let user1 = user1.clone();
                        let mod1 = mod1.clone();
                        let mod2 = mod2.clone();
                        tokio::spawn(async move {
                            let comment = s
                                .comment
                                .create_comment(
                                    user1.id,
                                    comment_system::model::CommentToCreate {
                                        content: format!(
                                            "Hi {}, comment to post from {}",
                                            number, &user1.alias
                                        ),
                                        content_type: fofo_utils::ContentType::Markdown,
                                        post_id: p.id,
                                        category_id: p.category_id,
                                        reply_user_id: p.created_by_id,
                                        reply_comment_id: 0,
                                        parent_id: 0,
                                        top_index: 0,
                                    },
                                )
                                .await
                                .unwrap();
                            if log {
                                info!("created comment id {}", &comment.id);
                            }
                            let comment = s
                                .comment
                                .create_comment(
                                    mod1.id,
                                    comment_system::model::CommentToCreate {
                                        content: format!(
                                            "Hi {}, reply comment {} from {}",
                                            number, comment.id, &mod1.alias
                                        ),
                                        content_type: fofo_utils::ContentType::Markdown,
                                        post_id: p.id,
                                        category_id: p.category_id,
                                        reply_user_id: comment.created_by_id,
                                        reply_comment_id: comment.id,
                                        parent_id: comment.id,
                                        top_index: 0,
                                    },
                                )
                                .await
                                .unwrap();
                            if log {
                                info!("created comment id {}", &comment.id);
                            }
                            let comment = s
                                .comment
                                .create_comment(
                                    mod2.id,
                                    comment_system::model::CommentToCreate {
                                        content: format!(
                                            "Hi {}, reply comment {} from {}",
                                            number, comment.id, &mod2.alias
                                        ),
                                        content_type: fofo_utils::ContentType::Markdown,
                                        post_id: p.id,
                                        category_id: p.category_id,
                                        reply_user_id: comment.created_by_id,
                                        reply_comment_id: comment.id,
                                        parent_id: comment.parent_id,
                                        top_index: 0,
                                    },
                                )
                                .await
                                .unwrap();
                            if log {
                                info!("created comment id {}", &comment.id);
                            }
                        })
                    })
                    .collect();
                for task in comments_task {
                    task.await.unwrap();
                }
            })
        })
        .collect();

    tasks1.append(&mut tasks2);
    for task in tasks1 {
        task.await.unwrap();
    }
    info!("initialized debug data....");
}
