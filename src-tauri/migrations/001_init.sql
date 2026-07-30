-- 动漫表
CREATE TABLE IF NOT EXISTS anime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    original_title TEXT,
    season INTEGER NOT NULL DEFAULT 1,
    subtitle_group TEXT,
    directory_path TEXT NOT NULL UNIQUE,
    cover_image TEXT,
    description TEXT,
    total_episodes INTEGER NOT NULL DEFAULT 0,
    watched_episodes INTEGER NOT NULL DEFAULT 0,
    is_movie INTEGER NOT NULL DEFAULT 0,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 剧集表
CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    episode_number INTEGER NOT NULL DEFAULT 1,
    duration REAL,
    file_size INTEGER,
    watched INTEGER NOT NULL DEFAULT 0,
    watch_progress REAL NOT NULL DEFAULT 0,
    last_watched_at DATETIME,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

-- 设置表
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_anime_title ON anime(title);
CREATE INDEX IF NOT EXISTS idx_anime_directory ON anime(directory_path);
CREATE INDEX IF NOT EXISTS idx_episodes_anime_id ON episodes(anime_id);
CREATE INDEX IF NOT EXISTS idx_episodes_file_path ON episodes(file_path);
