use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    static ref SEASON_REGEX: Regex = Regex::new(r"第(\d+)[季期]").unwrap();
    static ref BRACKET_CONTENT_REGEX: Regex = Regex::new(r"\[([^\]]+)\]").unwrap();
}

pub struct RenameOptions {
    pub include_subtitle_group: bool,
    pub backup_original_name: bool,
}

impl Default for RenameOptions {
    fn default() -> Self {
        Self {
            include_subtitle_group: true,
            backup_original_name: true,
        }
    }
}

pub struct ParsedDirectoryInfo {
    pub title: String,
    pub season: Option<i32>,
    pub subtitle_group: Option<String>,
    pub is_movie: bool,
    pub original_name: String,
}

pub fn parse_directory_name(dir_name: &str) -> ParsedDirectoryInfo {
    let original_name = dir_name.to_string();
    let mut title = dir_name.to_string();
    let mut season = None;
    let mut subtitle_group = None;
    let mut is_movie = false;

    let lower_name = title.to_lowercase();
    if lower_name.contains("剧场版")
        || lower_name.contains("movie")
        || lower_name.contains("ova")
        || lower_name.contains("oad")
    {
        is_movie = true;
    }

    if let Some(captures) = SEASON_REGEX.captures(&title) {
        if let Some(season_str) = captures.get(1) {
            if let Ok(s) = season_str.as_str().parse::<i32>() {
                season = Some(s);
            }
        }
    }

    let mut brackets: Vec<String> = Vec::new();
    for caps in BRACKET_CONTENT_REGEX.captures_iter(&title) {
        if let Some(content) = caps.get(1) {
            brackets.push(content.as_str().to_string());
        }
    }

    if let Some(last_bracket) = brackets.last() {
        if !last_bracket.contains('第')
            && !last_bracket.contains('季')
            && !SEASON_REGEX.is_match(last_bracket)
        {
            subtitle_group = Some(last_bracket.clone());
        }
    }

    title = SEASON_REGEX.replace_all(&title, "").to_string();
    title = BRACKET_CONTENT_REGEX.replace_all(&title, "").to_string();
    title = title.trim().to_string();

    if title.is_empty() {
        title = original_name.clone();
    }

    ParsedDirectoryInfo {
        title,
        season,
        subtitle_group,
        is_movie,
        original_name,
    }
}

pub fn generate_standard_name(info: &ParsedDirectoryInfo, options: &RenameOptions) -> String {
    let mut result = info.title.clone();

    if info.is_movie {
        result = format!("{} 剧场版", result);
    } else if let Some(season) = info.season {
        result = format!("{} 第{}季", result, season);
    } else {
        result = format!("{} 第1季", result);
    }

    if options.include_subtitle_group {
        if let Some(group) = &info.subtitle_group {
            result = format!("{} [{}]", result, group);
        }
    }

    result
}

pub fn rename_anime_directory(dir_path: &Path, options: &RenameOptions) -> Result<PathBuf> {
    let dir_name = dir_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

    let info = parse_directory_name(dir_name);
    let new_name = generate_standard_name(&info, options);

    if new_name == dir_name {
        return Ok(dir_path.to_path_buf());
    }

    let parent = dir_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;

    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return Err(anyhow::anyhow!(
            "目标目录已存在: {}",
            new_path.display()
        ));
    }

    if options.backup_original_name {
        let backup_file = dir_path.join("原目录名.txt");
        if !backup_file.exists() {
            std::fs::write(&backup_file, &info.original_name)
                .with_context(|| format!("Failed to write backup file: {:?}", backup_file))?;
        }
    }

    std::fs::rename(dir_path, &new_path)
        .with_context(|| format!("Failed to rename directory from {:?} to {:?}", dir_path, new_path))?;

    Ok(new_path)
}

pub fn organize_multi_season_anime(root_dir: &Path, options: &RenameOptions) -> Result<usize> {
    let mut organized = 0;
    let mut anime_groups: std::collections::HashMap<String, Vec<(PathBuf, ParsedDirectoryInfo)>> =
        std::collections::HashMap::new();

    for entry in std::fs::read_dir(root_dir).context("Failed to read root directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            let info = parse_directory_name(dir_name);
            anime_groups
                .entry(info.title.clone())
                .or_default()
                .push((path, info));
        }
    }

    for (title, items) in &anime_groups {
        if items.len() > 1 {
            let parent_dir = root_dir.join(title);

            if !parent_dir.exists() {
                std::fs::create_dir(&parent_dir)
                    .with_context(|| format!("Failed to create parent directory: {:?}", parent_dir))?;
            }

            for (path, info) in items {
                let new_name = generate_standard_name(info, options);
                let new_path = parent_dir.join(&new_name);

                if new_path.exists() {
                    continue;
                }

                if options.backup_original_name {
                    let backup_file = path.join("原目录名.txt");
                    if !backup_file.exists() {
                        if let Err(e) = std::fs::write(&backup_file, &info.original_name) {
                            eprintln!("Warning: Failed to write backup: {}", e);
                        }
                    }
                }

                if let Err(e) = std::fs::rename(path, &new_path) {
                    eprintln!("Warning: Failed to move {:?}: {}", path, e);
                } else {
                    organized += 1;
                }
            }
        }
    }

    Ok(organized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_name() {
        let info = parse_directory_name("一念永恒 第2季 [GM-Team]");
        assert_eq!(info.title, "一念永恒");
        assert_eq!(info.season, Some(2));
        assert_eq!(info.subtitle_group, Some("GM-Team".to_string()));
        assert!(!info.is_movie);
    }

    #[test]
    fn test_parse_movie_name() {
        let info = parse_directory_name("不死者之王 圣王国篇 剧场版 [DBD-Raws]");
        assert!(info.is_movie);
        assert_eq!(info.subtitle_group, Some("DBD-Raws".to_string()));
    }

    #[test]
    fn test_generate_standard_name() {
        let info = ParsedDirectoryInfo {
            title: "测试动画".to_string(),
            season: Some(2),
            subtitle_group: Some("TestGroup".to_string()),
            is_movie: false,
            original_name: "test".to_string(),
        };

        let name = generate_standard_name(&info, &RenameOptions::default());
        assert_eq!(name, "测试动画 第2季 [TestGroup]");
    }
}
