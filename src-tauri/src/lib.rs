pub mod commands;
pub mod db;
pub mod models;
pub mod organizer;
pub mod scanner;

use std::sync::Mutex;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            let db = tauri::async_runtime::block_on(async move {
                db::init_db(&app_data_dir)
                    .await
                    .expect("Failed to initialize database")
            });

            app.manage(Mutex::new(AppState { db }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_anime_list,
            commands::get_anime_episodes,
            commands::get_anime_by_title,
            commands::scan_library,
            commands::play_episode,
            commands::open_directory,
            commands::minimize_window,
            commands::toggle_maximize,
            commands::close_window,
            commands::toggle_episode_watched,
            commands::get_library_path,
            commands::set_library_path,
            commands::get_original_name,
            commands::delete_anime,
            commands::rename_anime_directory,
            commands::organize_multi_season,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
