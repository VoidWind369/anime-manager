pub mod anime;

use anyhow::{Context, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    Pool, Sqlite,
};
use std::{path::PathBuf, str::FromStr};

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(app_data_dir: &PathBuf) -> Result<DbPool> {
    std::fs::create_dir_all(app_data_dir).context("Failed to create app data directory")?;

    let db_path = app_data_dir.join("anime-manager.db");

    let options = SqliteConnectOptions::from_str(&format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    ))?
    .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    Ok(pool)
}
