use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::db::anime as db;
use crate::models::{Anime, Episode, ScanResult};
use crate::organizer;
use crate::scanner;

pub struct AppState {
    pub db: sqlx::SqlitePool,
}

fn get_pool(state: &State<'_, Mutex<AppState>>) -> sqlx::SqlitePool {
    state.lock().unwrap().db.clone()
}

#[tauri::command]
pub async fn get_anime_list(state: State<'_, Mutex<AppState>>) -> Result<Vec<Anime>, String> {
    let pool = get_pool(&state);
    db::get_anime_list(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_anime_episodes(
    state: State<'_, Mutex<AppState>>,
    anime_id: i64,
) -> Result<Vec<Episode>, String> {
    let pool = get_pool(&state);
    db::get_episodes_by_anime_id(&pool, anime_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_library(
    state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<ScanResult, String> {
    let pool = get_pool(&state);
    let result = scanner::scan_library(&pool, &path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn minimize_window(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_maximize(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn close_window(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .close()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_directory(path: String, _app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use tauri_plugin_shell::ShellExt;
        _app
            .shell()
            .open(&path, None)
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn play_episode(
    state: State<'_, Mutex<AppState>>,
    episode_id: i64,
    _app: AppHandle,
) -> Result<(), String> {
    let pool = get_pool(&state);
    let episode = db::get_episode_by_id(&pool, episode_id)
        .await
        .map_err(|e| e.to_string())?;

    let episode = episode.ok_or_else(|| "Episode not found".to_string())?;

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let path = &episode.file_path;
        Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use tauri_plugin_shell::ShellExt;
        _app
            .shell()
            .open(&episode.file_path, None)
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_episode_watched(
    state: State<'_, Mutex<AppState>>,
    episode_id: i64,
) -> Result<bool, String> {
    let pool = get_pool(&state);
    db::toggle_episode_watched(&pool, episode_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_library_path(state: State<'_, Mutex<AppState>>) -> Result<String, String> {
    let pool = get_pool(&state);
    let path = db::get_setting(&pool, "library_path")
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.unwrap_or_else(|| "E:\\动漫".to_string()))
}

#[tauri::command]
pub async fn set_library_path(
    state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<(), String> {
    let pool = get_pool(&state);
    db::set_setting(&pool, "library_path", &path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_anime_by_title(
    state: State<'_, Mutex<AppState>>,
    title: String,
) -> Result<Vec<Anime>, String> {
    let pool = get_pool(&state);
    db::get_anime_by_title(&pool, &title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_original_name(
    state: State<'_, Mutex<AppState>>,
    anime_id: i64,
) -> Result<String, String> {
    let pool = get_pool(&state);
    let anime = db::get_anime_by_id(&pool, anime_id)
        .await
        .map_err(|e| e.to_string())?;

    let anime = anime.ok_or_else(|| "Anime not found".to_string())?;

    if let Some(original) = anime.original_title {
        return Ok(original);
    }

    let dir_path = std::path::PathBuf::from(&anime.directory_path);
    let original_file = dir_path.join("原目录名.txt");

    if original_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&original_file) {
            return Ok(content.trim().to_string());
        }
    }

    Ok(String::new())
}

#[tauri::command]
pub async fn delete_anime(
    state: State<'_, Mutex<AppState>>,
    anime_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&state);
    db::delete_anime(&pool, anime_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_anime_directory(
    anime_id: i64,
    new_name: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let pool = get_pool(&state);

    let anime = db::get_anime_by_id(&pool, anime_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Anime not found".to_string())?;

    let old_path = std::path::PathBuf::from(&anime.directory_path);
    let parent = old_path
        .parent()
        .ok_or_else(|| "Invalid directory path".to_string())?;
    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return Err("目标目录已存在".to_string());
    }

    let backup_file = old_path.join("原目录名.txt");
    if !backup_file.exists() {
        if let Err(e) = std::fs::write(&backup_file, &anime.directory_path) {
            eprintln!("Warning: Failed to write backup: {}", e);
        }
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| format!("重命名失败: {}", e))?;

    let new_path_str = new_path.to_string_lossy().to_string();

    sqlx::query("UPDATE anime SET directory_path = ? WHERE id = ?")
        .bind(&new_path_str)
        .bind(anime_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("更新数据库失败: {}", e))?;

    let episodes = db::get_episodes_by_anime_id(&pool, anime_id)
        .await
        .map_err(|e| e.to_string())?;

    for ep in episodes {
        let old_ep_path = std::path::PathBuf::from(&ep.file_path);
        if let Ok(rel_path) = old_ep_path.strip_prefix(&old_path) {
            let new_ep_path = new_path.join(rel_path);
            let new_ep_path_str = new_ep_path.to_string_lossy().to_string();

            sqlx::query("UPDATE episodes SET file_path = ? WHERE id = ?")
                .bind(&new_ep_path_str)
                .bind(ep.id)
                .execute(&pool)
                .await
                .map_err(|e| format!("更新剧集路径失败: {}", e))?;
        }
    }

    Ok(new_path_str)
}

#[tauri::command]
pub async fn organize_multi_season(
    _state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<usize, String> {
    let options = organizer::RenameOptions::default();
    let root = std::path::PathBuf::from(&path);

    organizer::organize_multi_season_anime(&root, &options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_plugins_dir(app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("plugins");
    std::fs::create_dir_all(&dir).ok();
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_plugin_file(app: AppHandle, path: String) -> Result<Vec<u8>, String> {
    let plugins_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("plugins");
    let full_path = plugins_dir.join(&path);
    std::fs::read(&full_path).map_err(|e| format!("Failed to read {}: {}", path, e))
}


