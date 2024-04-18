use anyhow::{bail, Result};
use chrono::{Duration, Utc};
use fancy_regex::Regex;
use moka::future::{Cache, CacheBuilder};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::SqliteConnection;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    str::FromStr,
};
use tracing::error;

pub mod config;
pub mod meta;

use crate::config::SafeConfig;
use lazy_static::lazy_static;

#[allow(non_camel_case_types)]
pub type usizedb = u32;

lazy_static! {
    static ref IMAGE_URL_REGEX: Regex = Regex::new(r"!\[(?P<alt>.*?)\]\((?P<url>.*?)\)").unwrap();
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Clone, Copy, sqlx::Type)]
#[repr(u8)]
pub enum ContentType {
    Markdown,
}

impl ContentType {
    pub fn get_first_image_url(&self, content: &str) -> Option<String> {
        match self {
            ContentType::Markdown => {
                let caps = IMAGE_URL_REGEX.captures(content).unwrap();
                match caps {
                    Some(caps) => caps.name("url").map(|m| m.as_str().to_string()),
                    None => None,
                }
            }
        }
    }
}

const SPLIT_CHAR: &'static str = "~./*\\.~";
pub fn array_to_string<T: ToString>(arr: &[T]) -> String {
    let mut c = Vec::with_capacity(arr.len());
    for v in arr {
        c.push(v.to_string());
    }
    c.join(SPLIT_CHAR)
}

pub fn string_to_array<T: FromStr>(str: &str) -> Result<Vec<T>> {
    if str.len() == 0 {
        return Ok(vec![]);
    }
    let arr: Result<Vec<T>> = str
        .split(SPLIT_CHAR)
        .map(|s| match s.parse() {
            Ok(v) => Ok(v),
            Err(_) => bail!("Can't parse the value `{}` to target type.", s),
        })
        .collect();
    arr
}

pub fn calc_hash<T: Hash + Sized>(obj: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

pub async fn get_last_active_time(active_duration: Duration) -> usizedb {
    Utc::now()
        .checked_sub_signed(active_duration)
        .unwrap()
        .timestamp() as usizedb
}

pub async fn get_cache_instance<K, V>(config: SafeConfig) -> Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    use std::time::Duration;
    let max_capacity = config.cache_max_capacity;
    let ttl = config.ttl_seconds;
    let tti = config.tti_seconds;
    let mut builder: CacheBuilder<K, V, Cache<K, V>> =
        CacheBuilder::new(max_capacity).support_invalidation_closures();
    if ttl > 0 {
        builder = builder.time_to_live(Duration::from_secs(ttl));
    }
    if tti > 0 {
        builder = builder.time_to_idle(Duration::from_secs(tti));
    }
    builder.build()
}

pub async fn exists_table(tx: &mut SqliteConnection, table_name: &str) -> bool {
    // check if a table exists
    let query = r#"
        SELECT name FROM sqlite_master WHERE type='table' AND name=?;
    "#;

    match sqlx::query(query)
        .bind(table_name)
        .fetch_optional(&mut *tx)
        .await
        .unwrap()
    {
        Some(_row) => true,
        None => false,
    }
}

pub async fn increment_category_total_post(
    tx: &mut SqliteConnection,
    category_id: usizedb,
    count: usizedb,
) -> Result<()> {
    let r = sqlx::query("UPDATE categories SET total_post = total_post + ? WHERE id=?")
        .bind(count)
        .bind(category_id)
        .execute(&mut *tx)
        .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment total post!");
    }

    Ok(())
}

pub async fn increment_comment_total_sub_comments(
    tx: &mut SqliteConnection,
    id: usizedb,
    last_comment_at: usizedb,
    last_comment_by_id: usizedb,
    count: usizedb,
) -> Result<()> {
    let r = sqlx::query(
        "UPDATE comments SET 
        total_comment = total_comment + ?,
        last_comment_at = ?,
        last_comment_by_id = ? WHERE id = ?",
    )
    .bind(count)
    .bind(last_comment_at)
    .bind(last_comment_by_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment comment!");
    }

    Ok(())
}

pub async fn increment_post_total_comment(
    tx: &mut SqliteConnection,
    id: usizedb,
    last_comment_at: usizedb,
    last_comment_by_id: usizedb,
    total_comment: usizedb,
    total_comment_post: usizedb,
) -> Result<()> {
    let r = sqlx::query(
        "UPDATE posts SET 
        total_comment = total_comment + ?,
        total_comment_post = total_comment_post + ?,
        last_comment_at = ?,
        last_comment_by_id = ? WHERE id = ?",
    )
    .bind(total_comment)
    .bind(total_comment_post)
    .bind(last_comment_at)
    .bind(last_comment_by_id)
    .bind(id)
    .execute(tx)
    .await?;
    if r.rows_affected() == 0 {
        error!("Can't update row in table posts");
    }

    Ok(())
}

pub async fn increment_post_like(
    tx: &mut SqliteConnection,
    id: usizedb,
    likes: i64,
    dislikes: i64,
) -> Result<()> {
    let r = sqlx::query("UPDATE posts SET likes = likes + ?, dislikes = dislikes + ? WHERE id = ?")
        .bind(likes)
        .bind(dislikes)
        .bind(id)
        .execute(tx)
        .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment post likes and dislikes!");
    }

    Ok(())
}

pub async fn increment_comment_like(
    tx: &mut SqliteConnection,
    id: usizedb,
    likes: i64,
    dislikes: i64,
) -> Result<()> {
    let r = sqlx::query("UPDATE comments SET likes = likes + ?, dislikes = dislikes + ? WHERE id = ?")
        .bind(likes)
        .bind(dislikes)
        .bind(id)
        .execute(tx)
        .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment comment likes and dislikes!");
    }

    Ok(())
}

pub async fn increment_user_total_post(
    tx: &mut SqliteConnection,
    user_id: usizedb,
    count: usizedb,
) -> Result<()> {
    let r = sqlx::query("UPDATE users SET total_post = total_post + ? WHERE id=?")
        .bind(count)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment total post!");
    }

    Ok(())
}

pub async fn increment_user_total_comment(
    tx: &mut SqliteConnection,
    user_id: usizedb,
    count: usizedb,
) -> Result<()> {
    let r = sqlx::query("UPDATE users SET total_comment = total_comment + ? WHERE id=?")
        .bind(count)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    if r.rows_affected() == 0 {
        error!("Can't increment total post!");
    }

    Ok(())
}