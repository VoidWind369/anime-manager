use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Anime {
    pub id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub subtitle: Option<String>,
    pub season: i32,
    pub subtitle_group: Option<String>,
    pub directory_path: String,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub total_episodes: i32,
    pub watched_episodes: i32,
    pub is_movie: bool,
    pub added_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Episode {
    pub id: i64,
    pub anime_id: i64,
    pub title: String,
    pub file_path: String,
    pub episode_number: i32,
    pub duration: Option<f64>,
    pub watched: bool,
    pub watch_progress: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub total: usize,
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}


