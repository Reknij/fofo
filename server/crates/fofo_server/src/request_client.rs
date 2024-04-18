use std::pin::Pin;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use anyhow::Result;
use futures::Future;
use serde::{Deserialize, Serialize};

use user_system::model::UserInfo;

use super::api::SDW;

#[derive(Debug, Serialize, Deserialize)]
struct AuthQuery {
    pub auth: String,
}

pub struct RequestClient {
    user: Option<UserInfo>,
}

impl RequestClient {
    pub fn get_user_unwrap(&self) -> &UserInfo {
        self.user.as_ref().unwrap()
    }

    pub fn get_user(&self) -> Option<&UserInfo> {
        self.user.as_ref()
    }

    pub fn is_logined(&self) -> bool {
        self.user.is_some()
    }
}

impl FromRequest for RequestClient {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth = get_auth_from_request(req);
        let s = req.app_data::<SDW>().unwrap().clone();
        Box::pin(async move {
            let user = match auth {
                Some(auth) => match s.user.revert(&auth).await {
                    Ok(user) => user,
                    Err(_) => None,
                },
                None => None,
            };
            Ok(RequestClient { user })
        })
    }
}

pub fn get_auth_from_request(req: &HttpRequest) -> Option<String> {
    let auth_query = web::Query::<AuthQuery>::from_query(req.query_string());
    let auth = if let Ok(q) = auth_query {
        Some(q.auth.to_owned())
    } else if let Some(v) = req.headers().get("x-authorization") {
        v.to_str().ok().map(|v| v.to_owned())
    } else {
        None
    };
    auth
}
