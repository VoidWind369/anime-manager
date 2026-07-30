use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::{Anime, Episode};

pub async fn get_anime_list(pool: &SqlitePool) -> Result<Vec<Anime>> {
    let anime = sqlx::query_as::<_, Anime>(
        "SELECT id, title, original_title, subtitle, season, subtitle_group, directory_path,
         cover_image, description, total_episodes, watched_episodes,
         is_movie, added_at
         FROM anime ORDER BY title ASC"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch anime list")?;

    Ok(anime)
}

pub async fn get_anime_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Anime>> {
    let anime = sqlx::query_as::<_, Anime>(
        "SELECT id, title, original_title, subtitle, season, subtitle_group, directory_path,
         cover_image, description, total_episodes, watched_episodes,
         is_movie, added_at
         FROM anime WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch anime by id")?;

    Ok(anime)
}

pub async fn get_anime_by_path(pool: &SqlitePool, path: &str) -> Result<Option<Anime>> {
    let anime = sqlx::query_as::<_, Anime>(
        "SELECT id, title, original_title, subtitle, season, subtitle_group, directory_path,
         cover_image, description, total_episodes, watched_episodes,
         is_movie, added_at
         FROM anime WHERE directory_path = ?"
    )
    .bind(path)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch anime by path")?;

    Ok(anime)
}

pub async fn get_anime_by_title(pool: &SqlitePool, title: &str) -> Result<Vec<Anime>> {
    let anime = sqlx::query_as::<_, Anime>(
        "SELECT id, title, original_title, subtitle, season, subtitle_group, directory_path,
         cover_image, description, total_episodes, watched_episodes,
         is_movie, added_at
         FROM anime WHERE title = ? ORDER BY season ASC, subtitle_group ASC"
    )
    .bind(title)
    .fetch_all(pool)
    .await
    .context("Failed to fetch anime by title")?;

    Ok(anime)
}

pub async fn insert_anime(
    pool: &SqlitePool,
    title: &str,
    original_title: Option<&str>,
    subtitle: Option<&str>,
    season: i32,
    subtitle_group: Option<&str>,
    directory_path: &str,
    is_movie: bool,
) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO anime (title, original_title, subtitle, season, subtitle_group, directory_path, is_movie)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(title)
    .bind(original_title)
    .bind(subtitle)
    .bind(season)
    .bind(subtitle_group)
    .bind(directory_path)
    .bind(is_movie)
    .execute(pool)
    .await
    .context("Failed to insert anime")?;

    Ok(result.last_insert_rowid())
}

pub async fn update_anime_episode_count(pool: &SqlitePool, anime_id: i64) -> Result<()> {
    sqlx::query(
        "UPDATE anime SET total_episodes = (
            SELECT COUNT(*) FROM episodes WHERE anime_id = ?
        ),
        watched_episodes = (
            SELECT COUNT(*) FROM episodes WHERE anime_id = ? AND watched = 1
        ),
        updated_at = CURRENT_TIMESTAMP
        WHERE id = ?"
    )
    .bind(anime_id)
    .bind(anime_id)
    .bind(anime_id)
    .execute(pool)
    .await
    .context("Failed to update anime episode count")?;

    Ok(())
}

pub async fn delete_anime(pool: &SqlitePool, id: i64) -> Result<()> {
    sqlx::query("DELETE FROM anime WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete anime")?;

    Ok(())
}

pub async fn get_episodes_by_anime_id(pool: &SqlitePool, anime_id: i64) -> Result<Vec<Episode>> {
    let episodes = sqlx::query_as::<_, Episode>(
        "SELECT id, anime_id, title, file_path, episode_number, duration,
         watched, watch_progress
         FROM episodes WHERE anime_id = ? ORDER BY episode_number ASC"
    )
    .bind(anime_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch episodes")?;

    Ok(episodes)
}

pub async fn get_episode_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Episode>> {
    let episode = sqlx::query_as::<_, Episode>(
        "SELECT id, anime_id, title, file_path, episode_number, duration,
         watched, watch_progress
         FROM episodes WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch episode by id")?;

    Ok(episode)
}

pub async fn get_episode_by_path(pool: &SqlitePool, path: &str) -> Result<Option<Episode>> {
    let episode = sqlx::query_as::<_, Episode>(
        "SELECT id, anime_id, title, file_path, episode_number, duration,
         watched, watch_progress
         FROM episodes WHERE file_path = ?"
    )
    .bind(path)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch episode by path")?;

    Ok(episode)
}

pub async fn insert_episode(
    pool: &SqlitePool,
    anime_id: i64,
    title: &str,
    file_path: &str,
    episode_number: i32,
    duration: Option<f64>,
    file_size: Option<i64>,
) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO episodes (anime_id, title, file_path, episode_number, duration, file_size)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(anime_id)
    .bind(title)
    .bind(file_path)
    .bind(episode_number)
    .bind(duration)
    .bind(file_size)
    .execute(pool)
    .await
    .context("Failed to insert episode")?;

    Ok(result.last_insert_rowid())
}

pub async fn toggle_episode_watched(pool: &SqlitePool, episode_id: i64) -> Result<bool> {
    let episode = get_episode_by_id(pool, episode_id).await?;

    if let Some(ep) = episode {
        let new_watched = !ep.watched;
        sqlx::query(
            "UPDATE episodes SET watched = ?, last_watched_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(new_watched)
        .bind(episode_id)
        .execute(pool)
        .await
        .context("Failed to update episode watched status")?;

        update_anime_episode_count(pool, ep.anime_id).await?;

        Ok(new_watched)
    } else {
        Err(anyhow::anyhow!("Episode not found"))
    }
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let result: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .context("Failed to get setting")?;

    Ok(result.map(|r| r.0))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .context("Failed to set setting")?;

    Ok(())
}
