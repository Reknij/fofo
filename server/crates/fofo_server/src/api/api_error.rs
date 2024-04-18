use std::fmt::Display;

use actix_web::{http::StatusCode, ResponseError};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::error;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DetailErrorCode {
    InternalError = 10000,
    UnsupportedApi,
    IpAddressNotFound,
    TooManyRequests,
    FetchLimit,

    GetVerificationError = 10100,
    VerificationFailed,
    VerificationNotFound,

    LoginRequired = 10200,
    NoLoginRequired,
    NoPermission,
    AuthorizationRevertFailed,
    AuthorizationRequiredInUpdate,

    CreateUserFailed = 10300,
    UsernameAlreadyContain,
    UpdateUserFailed,
    UsernameNotFound,
    UserNotFound,
    PasswordNotMatch,

    PostNotFound = 10400,
    TagsExceedMaximum,

    CategoryNotFound = 10500,
    CategoryAlreadyContain,

    GroupAlreadyContain = 10600,
    GroupNotFound,

    SaveFileFailed = 10700,
    StaticFileNotFound,

    CommentNotFound = 10800,
    SamePostCommentRequired,
    ReplyCommentRequired,
    SameParentCommentRequired,
    ReplyCommentMissing,

    IllegalText = 10900,
    EmailAlreadyContain,
    TooManyTags,
    UneditableTime,

    BannedStatus = 11000,
    CategoryArchived,
    CategoryStopped,
    PostArchived,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailError {
    pub code: DetailErrorCode,
    pub msg: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub code: StatusCode,
    pub detail: DetailError,
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        error!("Invoke function got error:\n{}", value);
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            DetailErrorCode::InternalError,
            "Server got error when called service.",
        )
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "code {:?}: {}", &self.detail.code, &self.detail.msg)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let res = actix_web::HttpResponse::build(self.status_code()).json(&self.detail);
        res
    }
}

impl ApiError {
    pub fn new<T: ToString>(code: StatusCode, internal_code: DetailErrorCode, msg: T) -> Self {
        ApiError {
            code,
            detail: DetailError {
                code: internal_code,
                msg: msg.to_string(),
            },
        }
    }

    pub fn login_required() -> Self {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            DetailErrorCode::LoginRequired,
            "Please login to continue.",
        )
    }

    pub fn only_admin() -> Self {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            DetailErrorCode::NoPermission,
            "Only admin can access.",
        )
    }

    pub fn no_permission(msg: &str) -> Self {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            DetailErrorCode::NoPermission,
            msg,
        )
    }

    pub fn verification_failed() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::VerificationFailed,
            "Can't pass the captcha!",
        )
    }

    pub fn no_ip_address_found() -> Self {
        ApiError::new(
            StatusCode::EXPECTATION_FAILED,
            DetailErrorCode::IpAddressNotFound,
            "Can't get the client ip.",
        )
    }

    pub fn no_user_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::UserNotFound,
            "Please ensure user is exists.",
        )
    }

    pub fn no_post_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::PostNotFound,
            "Please ensure post is exists.",
        )
    }

    pub fn no_category_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::CategoryNotFound,
            "Please ensure category is exists.",
        )
    }

    pub fn no_group_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::GroupNotFound,
            "Please ensure group is exists.",
        )
    }

    pub fn no_static_file_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::StaticFileNotFound,
            "Please ensure file is exists.",
        )
    }

    pub fn no_comment_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::CommentNotFound,
            "Please ensure comment is exists.",
        )
    }

    pub fn no_verification_found() -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            DetailErrorCode::VerificationNotFound,
            "Please ensure verification is exists.",
        )
    }

    pub fn internal_error(msg: &str) -> Self {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            DetailErrorCode::InternalError,
            msg,
        )
    }

    pub fn password_not_match() -> Self {
        ApiError::new(
            StatusCode::EXPECTATION_FAILED,
            DetailErrorCode::PasswordNotMatch,
            "Password with user not match.",
        )
    }

    pub fn authorization_error() -> Self {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            DetailErrorCode::AuthorizationRevertFailed,
            "Authorization verify error. Please ensure your authorization is correct and not expired.",
        )
    }

    pub fn unique_username_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::UsernameAlreadyContain,
            "Username must unique.",
        )
    }

    pub fn unique_email_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::EmailAlreadyContain,
            "Email must unique.",
        )
    }

    pub fn unique_category_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::CategoryAlreadyContain,
            "Category must unique.",
        )
    }

    pub fn unique_group_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::GroupAlreadyContain,
            "Group must unique.",
        )
    }

    pub fn save_file_error() -> Self {
        ApiError::new(
            StatusCode::EXPECTATION_FAILED,
            DetailErrorCode::SaveFileFailed,
            "Can't save the file.",
        )
    }

    pub fn same_post_comment_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::SamePostCommentRequired,
            "Can't reply the another post's comment.",
        )
    }

    pub fn reply_comment_required() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::ReplyCommentRequired,
            "If not reply post, the reply comment id is required.",
        )
    }

    pub fn reply_comment_missing() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::ReplyCommentMissing,
            "Reply the missing comment.",
        )
    }

    pub fn illegal_email() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            "The email is illegal!",
        )
    }

    pub fn illegal_username() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            "The username is illegal!",
        )
    }

    pub fn illegal_password() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            "The password is illegal!",
        )
    }

    pub fn illegal_tag(tag: &str) -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            format!("The tag `{}` is illegal!", tag),
        )
    }

    pub fn illegal_title() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            "The title is illegal!",
        )
    }

    pub fn illegal_content() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::IllegalText,
            "The content is illegal!",
        )
    }

    pub fn too_many_tags() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::TooManyTags,
            "Too many tags!",
        )
    }

    pub fn uneditable_time() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::UneditableTime,
            "Uneditable time!",
        )
    }

    pub fn unsupported_api() -> Self {
        ApiError::new(
            StatusCode::METHOD_NOT_ALLOWED,
            DetailErrorCode::UnsupportedApi,
            "Unsupported api!",
        )
    }

    pub fn banned() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::BannedStatus,
            "Target object is banned.",
        )
    }

    pub fn post_archived() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::PostArchived,
            "Post is archived.",
        )
    }

    pub fn category_archived() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::CategoryArchived,
            "Category is archived.",
        )
    }

    pub fn category_stopped() -> Self {
        ApiError::new(
            StatusCode::NOT_ACCEPTABLE,
            DetailErrorCode::CategoryStopped,
            "Category is stopped.",
        )
    }

    pub fn too_many_requests(retry_time_str: &str) -> Self {
        ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            DetailErrorCode::TooManyRequests,
            format!("You request too many times. Please retry after {retry_time_str}"),
        )
    }

    pub fn fetch_limit() -> Self {
        ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            DetailErrorCode::FetchLimit,
            format!("Pagination limit maximum reached."),
        )
    }

    pub fn to_err<T>(self) -> Result<T, ApiError> {
        Err(self)
    }
}
