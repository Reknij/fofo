use std::time::Duration;
use anyhow::Result;
use fofo_utils::meta::SafeMetaInfo;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Sqlite};
use tokio::fs;

pub async fn initial(meta: SafeMetaInfo, once_time: bool) -> Result<sqlx::Pool<Sqlite>> {
    let path = meta.data_path.join("fofo.db");
    if once_time && path.is_file() {
        fs::remove_file(&path).await.unwrap();
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .busy_timeout(Duration::from_millis(6000))
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(12)
        .connect_with(options)
        .await?;
    Ok(pool)
}
