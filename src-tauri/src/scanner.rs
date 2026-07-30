use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::db::anime as db;
use crate::models::ScanResult;
use sqlx::SqlitePool;

lazy_static::lazy_static! {
    static ref SEASON_REGEX: Regex = Regex::new(r"第(\d+)[季期]").unwrap();
}

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "rmvb", "m4v", "ts",
];

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];

#[derive(Debug)]
struct ParsedAnimeDir {
    title: String,
    season: i32,
    subtitle_group: Option<String>,
    is_movie: bool,
    original_name: String,
}

fn parse_anime_dir_name(dir_name: &str) -> ParsedAnimeDir {
    let original_name = dir_name.to_string();
    let mut title = dir_name.to_string();
    let mut season = 1;
    let mut subtitle_group = None;
    let mut is_movie = false;

    if title.contains("剧场版")
        || title.contains("OVA")
        || title.contains("OAD")
        || title.contains("SP")
    {
        is_movie = true;
    }

    if let Some(captures) = SEASON_REGEX.captures(&title) {
        if let Some(season_str) = captures.get(1) {
            if let Ok(s) = season_str.as_str().parse::<i32>() {
                season = s;
            }
        }
    }

    // 提取字幕组：从后往前找第一个不是画质/版本标签的 [...] 内容
    let quality_keywords = [
        "WEB-DL", "WebRip", "WEBRip", "web-dl", "BDRip", "BD-Rip",
        "1080p", "720p", "2160p", "4K", "HEVC", "x264", "x265",
        "Ma10p", "10bit", "8bit", "FLAC", "AAC", "OPUS",
        "简繁内封", "简繁", "繁中", "简中",
        "DVD", "BluRay", "REMUX",
    ];

    fn is_quality_tag(content: &str, keywords: &[&str]) -> bool {
        let upper = content.to_uppercase();
        for kw in keywords {
            if upper.contains(&kw.to_uppercase()) {
                return true;
            }
        }
        false
    }

    // 收集所有 [...] 块
    let mut brackets: Vec<(usize, usize, String)> = Vec::new();
    let chars: Vec<char> = title.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != ']' {
                j += 1;
            }
            if j < chars.len() {
                let content: String = chars[i + 1..j].iter().collect();
                brackets.push((i, j, content));
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }

    // 从后往前找第一个不是画质标签的作为字幕组
    for (_start, _end, content) in brackets.iter().rev() {
        if !is_quality_tag(content, &quality_keywords) {
            subtitle_group = Some(content.clone());
            break;
        }
    }

    title = SEASON_REGEX.replace_all(&title, "").to_string();

    // 移除所有 [...] 块
    let mut cleaned = String::new();
    let mut in_bracket = false;
    for ch in title.chars() {
        if ch == '[' {
            in_bracket = true;
        } else if ch == ']' {
            in_bracket = false;
        } else if !in_bracket {
            cleaned.push(ch);
        }
    }
    title = cleaned;

    // 移除开头的"剧场版"
    if title.starts_with("剧场版") {
        title = title.trim_start_matches("剧场版").to_string();
    }
    // 移除结尾的"剧场版"
    if title.ends_with("剧场版") {
        title = title.trim_end_matches("剧场版").to_string();
    }
    // 移除中间的" 剧场版 "或"剧场版 "等
    title = title.replace(" 剧场版 ", " ");
    title = title.replace("剧场版 ", "");
    title = title.replace(" 剧场版", "");

    // 移除 OVA/OAD/SP 关键词
    for kw in &["OVA", "OAD", "SP"] {
        title = title.replace(&format!(" {} ", kw), " ");
        title = title.replace(&format!("{} ", kw), "");
        title = title.replace(&format!(" {}", kw), "");
        if title.starts_with(kw) {
            title = title.trim_start_matches(kw).to_string();
        }
        if title.ends_with(kw) {
            title = title.trim_end_matches(kw).to_string();
        }
    }

    title = title.trim().to_string();

    // 处理"柯里乌斯之梦"这种番外篇——如果标题包含主标题 + 空格 + 副标题，
    // 且主标题是已有的番剧名，保留完整标题但 season=1, is_movie=true
    // （前端按 title 分组，所以番外篇 title 需要和正片一致）
    // 这里简单处理：如果标题包含"之梦"、"篇"等且有空格分隔，
    // 提取空格前的作为主标题
    // 实际上更好的做法是保持完整标题，前端按"主标题"分组
    // 暂时保留完整标题，后续再优化分组逻辑

    if title.is_empty() {
        title = original_name.clone();
    }

    ParsedAnimeDir {
        title,
        season,
        subtitle_group,
        is_movie,
        original_name,
    }
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn find_cover_image(dir_path: &Path) -> Option<String> {
    let candidates = ["cover", "poster", "folder", "封面"];

    for candidate in candidates {
        for ext in IMAGE_EXTENSIONS {
            let cover_path = dir_path.join(format!("{}.{}", candidate, ext));
            if cover_path.exists() {
                return Some(cover_path.to_string_lossy().to_string());
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                            if !name.starts_with('.') {
                                return Some(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn read_original_name(dir_path: &Path) -> Option<String> {
    let original_file = dir_path.join("原目录名.txt");
    if original_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&original_file) {
            let content = content.trim().to_string();
            if !content.is_empty() {
                return Some(content);
            }
        }
    }
    None
}

/// 只检查目录直接子层是否有视频文件（不递归子目录）
fn dir_has_direct_video_files(dir_path: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() && is_video_file(&entry.path()) {
                    return true;
                }
            }
        }
    }
    false
}

/// 检查目录是否有视频文件（含一层子目录，用于判断子目录是否为动漫目录）
fn dir_has_video_files(dir_path: &Path) -> bool {
    if dir_has_direct_video_files(dir_path) {
        return true;
    }
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if dir_has_direct_video_files(&entry.path()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn find_anime_directories(root_path: &Path) -> Result<Vec<(PathBuf, Option<String>)>> {
    let mut dirs: Vec<(PathBuf, Option<String>)> = Vec::new();

    if !root_path.exists() {
        eprintln!("Root path does not exist: {:?}", root_path);
        return Ok(dirs);
    }

    let entries = std::fs::read_dir(root_path).context("Failed to read root directory")?;
    
    for (idx, entry) in entries.enumerate() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry {}: {}", idx, e);
                continue;
            }
        };
        
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        
        if dir_name.starts_with('.') {
            continue;
        }

        eprintln!("Scanning directory [{}/?]: {}", idx, dir_name);

        // 直接子层有视频文件 → 独立动漫目录
        if dir_has_direct_video_files(&path) {
            dirs.push((path, None));
            continue;
        }

        let inner_entries = match std::fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut sub_anime_dirs: Vec<PathBuf> = Vec::new();
        let mut has_non_anime_subdir = false;
        
        for inner_entry in inner_entries.filter_map(|e| e.ok()) {
            let inner_path = inner_entry.path();
            if !inner_path.is_dir() {
                continue;
            }
            
            let inner_name = inner_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if inner_name.starts_with('.') {
                continue;
            }

            if dir_has_video_files(&inner_path) {
                sub_anime_dirs.push(inner_path);
            } else {
                has_non_anime_subdir = true;
                continue;
            }
        }

        if !sub_anime_dirs.is_empty() && !has_non_anime_subdir {
            let parent_name = dir_name.to_string();
            for sub_dir in sub_anime_dirs {
                dirs.push((sub_dir, Some(parent_name.clone())));
            }
        } else if !sub_anime_dirs.is_empty() && has_non_anime_subdir {
            for sub_dir in sub_anime_dirs {
                dirs.push((sub_dir, None));
            }
        }
    }

    eprintln!("Found {} anime directories", dirs.len());
    Ok(dirs)
}

fn find_video_files(dir_path: &Path) -> Result<Vec<PathBuf>> {
    let mut videos = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() && is_video_file(&entry.path()) {
                    videos.push(entry.path().to_path_buf());
                } else if file_type.is_dir() {
                    if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                        for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                            if let Ok(sub_type) = sub_entry.file_type() {
                                if sub_type.is_file() && is_video_file(&sub_entry.path()) {
                                    videos.push(sub_entry.path().to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    videos.sort_by(|a, b| {
        let a_name = a.file_name().unwrap_or_default();
        let b_name = b.file_name().unwrap_or_default();
        alphanumeric_sort(a_name.to_string_lossy().as_ref(), b_name.to_string_lossy().as_ref())
    });

    Ok(videos)
}

fn alphanumeric_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut i = 0;
    let mut j = 0;

    while i < a_chars.len() && j < b_chars.len() {
        if a_chars[i].is_ascii_digit() && b_chars[j].is_ascii_digit() {
            let mut a_num = 0i64;
            let mut b_num = 0i64;

            while i < a_chars.len() && a_chars[i].is_ascii_digit() {
                a_num = a_num * 10 + (a_chars[i] as i64 - '0' as i64);
                i += 1;
            }

            while j < b_chars.len() && b_chars[j].is_ascii_digit() {
                b_num = b_num * 10 + (b_chars[j] as i64 - '0' as i64);
                j += 1;
            }

            if a_num != b_num {
                return a_num.cmp(&b_num);
            }
        } else {
            match a_chars[i].cmp(&b_chars[j]) {
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
                other => return other,
            }
        }
    }

    a_chars.len().cmp(&b_chars.len())
}

pub async fn scan_library(pool: &SqlitePool, root_path: &str) -> Result<ScanResult> {
    let root = PathBuf::from(root_path);
    let anime_dirs = find_anime_directories(&root)?;

    let mut total = 0;
    let mut added = 0;
    let mut updated = 0;

    let mut scanned_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (dir_path, group_title) in &anime_dirs {
        let dir_name = dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let parsed = parse_anime_dir_name(dir_name);
        let dir_path_str = dir_path.to_string_lossy().to_string();
        scanned_paths.insert(dir_path_str.clone());

        let (title, subtitle) = if let Some(group) = group_title {
            let sub = if parsed.title != *group && !parsed.title.is_empty() {
                Some(parsed.title.clone())
            } else {
                None
            };
            (group.clone(), sub)
        } else {
            (parsed.title.clone(), None)
        };

        let existing = db::get_anime_by_path(pool, &dir_path_str).await?;

        let anime_id = if let Some(anime) = existing {
            updated += 1;
            if anime.title != title || anime.subtitle.as_deref() != subtitle.as_deref() {
                sqlx::query("UPDATE anime SET title = ?, subtitle = ? WHERE id = ?")
                    .bind(&title)
                    .bind(&subtitle)
                    .bind(anime.id)
                    .execute(pool)
                    .await?;
            }
            anime.id
        } else {
            added += 1;
            let id = db::insert_anime(
                pool,
                &title,
                read_original_name(dir_path).as_deref().or(Some(&parsed.original_name)),
                subtitle.as_deref(),
                parsed.season,
                parsed.subtitle_group.as_deref(),
                &dir_path_str,
                parsed.is_movie,
            )
            .await?;

            if let Some(cover) = find_cover_image(dir_path) {
                sqlx::query("UPDATE anime SET cover_image = ? WHERE id = ?")
                    .bind(cover)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }

            id
        };

        let video_files = match find_video_files(dir_path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error scanning videos in {:?}: {}", dir_path, e);
                continue;
            }
        };

        if video_files.is_empty() {
            db::update_anime_episode_count(pool, anime_id).await?;
            total += 1;
            continue;
        }

        // 只查询当前 anime 的 episodes（用于更新和清理）
        let existing_ep_paths: std::collections::HashSet<String> = match sqlx::query_as::<_, (String,)>(
            "SELECT file_path FROM episodes WHERE anime_id = ?"
        )
        .bind(anime_id)
        .fetch_all(pool)
        .await {
            Ok(rows) => rows.into_iter().map(|(p,)| p).collect(),
            Err(e) => {
                eprintln!("Error fetching existing episodes for anime {}: {}", anime_id, e);
                std::collections::HashSet::new()
            }
        };

        let mut ep_index = 0;
        for video_path in &video_files {
            ep_index += 1;
            let file_path_str = video_path.to_string_lossy().to_string();
            let filename = video_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ep_title = Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(filename)
                .to_string();

            let ep_number = ep_index as i32;

            if existing_ep_paths.contains(&file_path_str) {
                // episode 已属于当前 anime，更新 episode_number
                sqlx::query("UPDATE episodes SET episode_number = ? WHERE file_path = ?")
                    .bind(ep_number)
                    .bind(&file_path_str)
                    .execute(pool)
                    .await?;
            } else {
                let final_title = if video_files.len() == 1 && parsed.is_movie {
                    parsed.title.clone()
                } else {
                    ep_title.clone()
                };

                let file_size = std::fs::metadata(video_path).ok().map(|m| m.len() as i64);

                // 先尝试插入，如果 file_path 已存在（属于其他 anime），则更新其 anime_id
                let insert_result = sqlx::query(
                    "INSERT INTO episodes (anime_id, title, file_path, episode_number, file_size) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(anime_id)
                .bind(&final_title)
                .bind(&file_path_str)
                .bind(ep_number)
                .bind(file_size)
                .execute(pool)
                .await;

                if let Err(_) = insert_result {
                    // file_path 已存在（属于其他 anime），更新归属
                    sqlx::query("UPDATE episodes SET anime_id = ?, episode_number = ?, title = ? WHERE file_path = ?")
                        .bind(anime_id)
                        .bind(ep_number)
                        .bind(&final_title)
                        .bind(&file_path_str)
                        .execute(pool)
                        .await?;
                }
            }
        }

        let current_ep_paths: std::collections::HashSet<String> = video_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // 只清理当前 anime 下不再存在的 episodes
        for old_path in &existing_ep_paths {
            if !current_ep_paths.contains(old_path) {
                sqlx::query("DELETE FROM episodes WHERE file_path = ? AND anime_id = ?")
                    .bind(old_path)
                    .bind(anime_id)
                    .execute(pool)
                    .await?;
            }
        }

        db::update_anime_episode_count(pool, anime_id).await?;
        total += 1;
    }

    let mut removed = 0;
    let stale_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, directory_path FROM anime"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch anime for cleanup")?;

    for (anime_id, dir_path) in &stale_rows {
        if scanned_paths.contains(dir_path) {
            continue;
        }
        if Path::new(dir_path).exists() {
            continue;
        }

        let tracked_ids: Vec<i64> = sqlx::query_as("SELECT id FROM tracked_anime WHERE anime_id = ?")
            .bind(anime_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(id,)| id)
            .collect();

        for tid in &tracked_ids {
            let _ = sqlx::query("DELETE FROM update_history WHERE tracked_anime_id = ?")
                .bind(tid)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM tracked_anime WHERE anime_id = ?")
            .bind(anime_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM episodes WHERE anime_id = ?")
            .bind(anime_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM anime WHERE id = ?")
            .bind(anime_id)
            .execute(pool)
            .await;
        removed += 1;
    }

    Ok(ScanResult {
        total,
        added,
        updated,
        removed,
    })
}
