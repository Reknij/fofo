use std::collections::HashMap;

use anyhow::Result;
use category_system::model::Category;
use chrono::Utc;
use comment_system::model::{CommentBaseInfo, CommentInfo};
use fancy_regex::Regex;
use fofo_utils::usizedb;
use group_system::model::Group;
use like_system::model::{
    LikeStatus,
    LikeStatusFlag::{TargetComment, TargetPost},
};
use post_system::model::{PostBaseInfo, PostInfo};
use user_system::model::{SafeUserInfo, UserInfo};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use crate::ServerData;

use super::{api_error::ApiError, verification_controller::model::VerificationKey};

pub struct LegalityVerification;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationTargetWrapper<T> {
    pub target: T,
    pub verification: Option<super::verification_controller::model::VerificationKey>,
}

pub trait Verify<V> {
    fn verify(&self) -> V;
}

pub struct GetDatasExtendedBuilder<'a, T> {
    data_extended: GetDatasExtended<T>,
    s: &'a ServerData,
}

impl<'a, T> GetDatasExtendedBuilder<'a, T> {
    pub fn new(s: &'a ServerData) -> Self {
        Self {
            data_extended: GetDatasExtended::empty(),
            s,
        }
    }
    pub fn build(self) -> GetDatasExtended<T> {
        self.data_extended
    }
    pub fn set_data(mut self, data: ListSlice<T>) -> Self {
        self.data_extended.set_data(data);
        self
    }

    pub async fn extend_posts(
        &mut self,
        tx: &mut SqliteConnection,
        ids: Vec<usizedb>,
    ) -> Result<&mut Self> {
        for id in ids {
            let id = id.to_owned();
            let value = self.s.post.get_post(tx, id, false).await?;
            if let Some(v) = value {
                if !self.data_extended.get_posts_map().contains_key(&id) {
                    self.extend_users(
                        tx,
                        vec![v.created_by_id, v.last_edit_by_id, v.last_comment_by_id],
                    )
                    .await?;
                    self.extend_categories(tx, vec![v.category_id]).await?;
                    self.data_extended.get_posts_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_comments(
        &mut self,
        tx: &mut SqliteConnection,
        ids: Vec<usizedb>,
    ) -> Result<&mut Self> {
        for id in ids {
            let id = id.to_owned();
            let value = self.s.comment.get_comment(tx, id).await?;
            if let Some(v) = value {
                if !self.data_extended.get_comments_map().contains_key(&id) {
                    let reply_comment = self.s.comment.get_comment(tx, v.reply_comment_id).await?;
                    if let Some(v) = reply_comment {
                        if !self.data_extended.get_comments_map().contains_key(&id) {
                            self.extend_users(
                                tx,
                                vec![v.created_by_id, v.last_edit_by_id, v.last_comment_by_id],
                            )
                            .await?;
                            self.data_extended.get_comments_map().insert(id, v);
                        }
                    }
                    self.extend_users(
                        tx,
                        vec![v.created_by_id, v.last_edit_by_id, v.last_comment_by_id],
                    )
                    .await?;
                    self.extend_posts(tx, vec![v.post_id]).await?;
                    self.data_extended.get_comments_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_categories(
        &mut self,
        tx: &mut SqliteConnection,
        ids: Vec<usizedb>,
    ) -> Result<&mut Self> {
        for id in ids {
            let id = id.to_owned();
            let value = self.s.category.get_category(tx, id).await?;
            if let Some(v) = value {
                if !self.data_extended.get_categories_map().contains_key(&id) {
                    self.extend_groups(tx, v.group_ids.to_owned()).await?;
                    self.extend_users(tx, v.moderator_ids.to_owned()).await?;
                    self.data_extended.get_categories_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_groups(
        &mut self,
        tx: &mut SqliteConnection,
        ids: Vec<usizedb>,
    ) -> Result<&mut Self> {
        for id in ids {
            let id = id.to_owned();
            let value = self.s.group.get_group(tx, id).await?;
            if let Some(v) = value {
                if !self.data_extended.get_groups_map().contains_key(&id) {
                    self.data_extended.get_groups_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_users(
        &mut self,
        tx: &mut SqliteConnection,
        ids: Vec<usizedb>,
    ) -> Result<&mut Self> {
        for id in ids {
            let id = id.to_owned();
            let value = self.s.user.get_safe_user(tx, id).await?;
            if let Some(v) = value {
                if !self.data_extended.get_users_map().contains_key(&id) {
                    self.extend_groups(tx, v.group_ids.to_owned()).await?;

                    self.data_extended.get_users_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_posts_like_status(
        &mut self,
        tx: &mut SqliteConnection,
        post_ids: Vec<usizedb>,
        user_id: usizedb,
    ) -> Result<&mut Self> {
        for id in post_ids {
            let value = self
                .s
                .like
                .get_like_status(tx, id, TargetPost, user_id)
                .await?;
            if let Some(v) = value {
                if !self
                    .data_extended
                    .get_posts_like_status_map()
                    .contains_key(&id)
                {
                    self.data_extended.get_posts_like_status_map().insert(id, v);
                }
            }
        }

        Ok(self)
    }

    pub async fn extend_comments_like_status(
        &mut self,
        tx: &mut SqliteConnection,
        comment_ids: Vec<usizedb>,
        user_id: usizedb,
    ) -> Result<&mut Self> {
        for id in comment_ids {
            let value = self
                .s
                .like
                .get_like_status(tx, id, TargetComment, user_id)
                .await?;
            if let Some(v) = value {
                if !self
                    .data_extended
                    .get_comments_like_status_map()
                    .contains_key(&id)
                {
                    self.data_extended
                        .get_comments_like_status_map()
                        .insert(id, v);
                }
            }
        }

        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDatasExtended<T> {
    pub data: ListSlice<T>,
    pub posts: Option<HashMap<usizedb, PostInfo>>,
    pub comments: Option<HashMap<usizedb, CommentInfo>>,
    pub categories: Option<HashMap<usizedb, Category>>,
    pub groups: Option<HashMap<usizedb, Group>>,
    pub users: Option<HashMap<usizedb, SafeUserInfo>>,
    pub posts_like_status: Option<HashMap<usizedb, LikeStatus>>,
    pub comments_like_status: Option<HashMap<usizedb, LikeStatus>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSlice<T> {
    pub items: Vec<T>,
    pub total: usizedb,
}

impl<T> ListSlice<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total: 0,
        }
    }
}

impl<T> GetDatasExtended<T> {
    pub fn empty() -> Self {
        GetDatasExtended {
            data: ListSlice::empty(),
            posts: None,
            comments: None,
            categories: None,
            groups: None,
            users: None,
            posts_like_status: None,
            comments_like_status: None,
        }
    }

    pub fn set_data(&mut self, data: ListSlice<T>) -> &mut Self {
        self.data = data;
        self
    }

    fn get_categories_map(&mut self) -> &mut HashMap<usizedb, Category> {
        if self.categories.is_none() {
            self.categories = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.categories.as_mut().unwrap_unchecked() }
    }

    fn get_groups_map(&mut self) -> &mut HashMap<usizedb, Group> {
        if self.groups.is_none() {
            self.groups = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.groups.as_mut().unwrap_unchecked() }
    }

    fn get_users_map(&mut self) -> &mut HashMap<usizedb, SafeUserInfo> {
        if self.users.is_none() {
            self.users = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.users.as_mut().unwrap_unchecked() }
    }

    fn get_posts_map(&mut self) -> &mut HashMap<usizedb, PostInfo> {
        if self.posts.is_none() {
            self.posts = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.posts.as_mut().unwrap_unchecked() }
    }

    fn get_comments_map(&mut self) -> &mut HashMap<usizedb, CommentInfo> {
        if self.comments.is_none() {
            self.comments = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.comments.as_mut().unwrap_unchecked() }
    }

    fn get_posts_like_status_map(&mut self) -> &mut HashMap<usizedb, LikeStatus> {
        if self.posts_like_status.is_none() {
            self.posts_like_status = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.posts_like_status.as_mut().unwrap_unchecked() }
    }

    fn get_comments_like_status_map(&mut self) -> &mut HashMap<usizedb, LikeStatus> {
        if self.comments_like_status.is_none() {
            self.comments_like_status = Some(HashMap::with_capacity(self.data.items.len()));
        }

        unsafe { self.comments_like_status.as_mut().unwrap_unchecked() }
    }
}

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,4}$").unwrap();
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-z0-9]{5,}$").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^(?=.*[a-zA-Z])(?=.*\d)[!-~]{8,128}$").unwrap();
    static ref TITLE_AND_TAG_REGEX: Regex = Regex::new(r"^\S.{0,128}$").unwrap();
}

impl LegalityVerification {
    pub fn is_email(v: &str) -> bool {
        EMAIL_REGEX.is_match(v).unwrap()
    }

    pub fn is_username(v: &str) -> bool {
        USERNAME_REGEX.is_match(v).unwrap()
    }

    pub fn is_password(v: &str) -> bool {
        PASSWORD_REGEX.is_match(v).unwrap()
    }

    pub fn is_title(v: &str) -> bool {
        TITLE_AND_TAG_REGEX.is_match(v).unwrap()
    }

    pub fn is_content(v: &str) -> bool {
        v.len() <= 65535
    }

    pub fn is_tag(v: &str) -> bool {
        TITLE_AND_TAG_REGEX.is_match(v).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum WhatToDo {
    WritePost,
    WriteComment,
    ReplyComment,
    LikePost,
    LikeComment,
    None,
}

pub async fn check_user(
    s: &ServerData,
    tx: &mut SqliteConnection,
    user: Option<&UserInfo>,
    w: WhatToDo,
) -> Result<(), ApiError> {
    if let Some(user) = user {
        for gid in &user.group_ids {
            check_group(&s, tx, *gid, w).await?;
        }
        match user.status {
            user_system::model::UserStatus::Active
            | user_system::model::UserStatus::Observer => (),
            user_system::model::UserStatus::Banned => {
                return ApiError::banned().to_err();
            }
            user_system::model::UserStatus::OnlyComment => {
                if w == WhatToDo::WritePost {
                    return ApiError::no_permission().to_err();
                }
            }
        }
    }

    Ok(())
}

pub async fn check_group(
    s: &ServerData,
    tx: &mut SqliteConnection,
    id: usizedb,
    w: WhatToDo,
) -> Result<(), ApiError> {
    if !s.group.is_exists(tx, id).await? {
        return ApiError::no_group_found().to_err();
    }
    match s.group.get_status(tx, id).await? {
        group_system::model::GroupStatus::Active => (),
        group_system::model::GroupStatus::OnlyComment => {
            if w == WhatToDo::WritePost {
                return ApiError::no_permission().to_err();
            }
        }
        group_system::model::GroupStatus::Observer => {
            return ApiError::no_permission().to_err();
        }
    }

    Ok(())
}

pub async fn check_post(
    s: &ServerData,
    tx: &mut SqliteConnection,
    post_id: usizedb,
    user: Option<&UserInfo>,
    w: WhatToDo,
) -> Result<PostBaseInfo, ApiError> {
    if !s.post.is_exists(tx, post_id).await? {
        return ApiError::no_post_found().to_err();
    }

    let post = s.post.get_post_base(tx, post_id).await?;
    check_category(s, tx, post.category_id, user, w).await?;

    match s.post.get_status(tx, post_id).await? {
        post_system::model::PostStatus::Active => {
            if w == WhatToDo::WritePost {
                let editable_seconds = s.core.get_config().editable_seconds as usizedb;
                let manage = s.category.can_manage(tx, post.category_id, user).await?;
                if !manage && post.created_at + editable_seconds < Utc::now().timestamp() as usizedb
                {
                    return ApiError::uneditable_time().to_err();
                }
                if user.is_none() || (post.created_by_id != user.unwrap().id && !manage) {
                    return ApiError::no_permission().to_err();
                }
            }
        }
        post_system::model::PostStatus::Archived => {
            if w != WhatToDo::None && !s.category.can_manage(tx, post.category_id, user).await? {
                return ApiError::post_archived().to_err();
            }
        }
        post_system::model::PostStatus::Banned => {
            if w != WhatToDo::None && !s.category.can_manage(tx, post.category_id, user).await? {
                return ApiError::banned().to_err();
            }
        }
    }
    Ok(post)
}

pub async fn check_comment(
    s: &ServerData,
    tx: &mut SqliteConnection,
    comment_id: usizedb,
    user: Option<&UserInfo>,
    w: WhatToDo,
) -> Result<(PostBaseInfo, CommentBaseInfo), ApiError> {
    if !s.comment.is_exists(tx, comment_id).await? {
        return match w {
            WhatToDo::WriteComment | WhatToDo::LikeComment => ApiError::no_comment_found().to_err(),
            WhatToDo::ReplyComment => ApiError::reply_comment_missing().to_err(),
            WhatToDo::LikePost | WhatToDo::WritePost | WhatToDo::None => {
                panic!("Check commant but provide other action type.")
            }
        };
    }
    let comment = s.comment.get_comment_base(tx, comment_id).await?;
    let post = check_post(s, tx, comment.post_id, user, w).await?;
    match s.comment.get_status(tx, comment_id).await? {
        comment_system::model::CommentStatus::Active => {
            if w == WhatToDo::WriteComment {
                let editable_seconds = s.core.get_config().editable_seconds as usizedb;
                let manage = s.category.can_manage(tx, post.category_id, user).await?;
                if !manage
                    && comment.created_at + editable_seconds < Utc::now().timestamp() as usizedb
                {
                    return ApiError::uneditable_time().to_err();
                }
                if user.is_none() || (comment.created_by_id != user.unwrap().id && !manage) {
                    return ApiError::no_permission().to_err();
                }
            }
        }
        comment_system::model::CommentStatus::Banned => {
            if w != WhatToDo::None && !s.category.can_manage(tx, comment.category_id, user).await? {
                return ApiError::banned().to_err();
            }
        }
    }
    Ok((post, comment))
}

pub async fn check_category(
    s: &ServerData,
    tx: &mut SqliteConnection,
    category_id: usizedb,
    user: Option<&UserInfo>,
    w: WhatToDo,
) -> Result<(), ApiError> {
    if !s.category.is_exists(tx, category_id).await? {
        return ApiError::no_category_found().to_err();
    }

    match s.category.get_status(tx, category_id).await? {
        category_system::model::CategoryStatus::Active => match w {
            WhatToDo::LikePost | WhatToDo::LikeComment | WhatToDo::WritePost => {
                if !s.category.can_write(tx, category_id, user).await? {
                    return ApiError::no_permission().to_err();
                }
            }
            WhatToDo::WriteComment | WhatToDo::ReplyComment => {
                if !s.category.can_comment(tx, category_id, user).await? {
                    return ApiError::no_permission().to_err();
                }
            }
            WhatToDo::None => (),
        },
        category_system::model::CategoryStatus::Archived => {
            if w != WhatToDo::None && !s.category.can_manage(tx, category_id, user).await? {
                return ApiError::category_archived().to_err();
            }
        }
        category_system::model::CategoryStatus::Stopped => {
            if user.is_none() || !user.unwrap().is_admin() {
                return ApiError::category_stopped().to_err();
            }
        }
    }

    Ok(())
}

pub async fn check_verification_and_pass_it(
    s: &ServerData,
    tx: &mut SqliteConnection,
    vkey: Option<&VerificationKey>,
) -> Result<(), ApiError> {
    let vkey = if vkey.is_none() {
        return Ok(());
    } else {
        vkey.unwrap()
    };
    let result = s
        .verification
        .set_verification_passed(tx, vkey.verification_id, &vkey.secret_key)
        .await?;
    match result {
        Some(result) => {
            if !result {
                ApiError::verification_failed().to_err()
            } else {
                Ok(())
            }
        }
        None => ApiError::no_verification_found().to_err(),
    }
}

pub async fn can_manage_post(
    s: &ServerData,
    tx: &mut SqliteConnection,
    post_id: usizedb,
    user: Option<&UserInfo>,
) -> Result<(), ApiError> {
    if !s.post.is_exists(tx, post_id).await? {
        return ApiError::no_post_found().to_err();
    }
    let post = s.post.get_post_base(tx, post_id).await?;
    if !s.category.can_manage(tx, post.category_id, user).await? {
        ApiError::no_permission().to_err()
    } else {
        Ok(())
    }
}

pub async fn can_manage_comment(
    s: &ServerData,
    tx: &mut SqliteConnection,
    comment_id: usizedb,
    user: Option<&UserInfo>,
) -> Result<(), ApiError> {
    if !s.comment.is_exists(tx, comment_id).await? {
        return ApiError::no_comment_found().to_err();
    }
    let comment = s.comment.get_comment_base(tx, comment_id).await?;
    if !s.category.can_manage(tx, comment.category_id, user).await? {
        ApiError::no_permission().to_err()
    } else {
        Ok(())
    }
}
