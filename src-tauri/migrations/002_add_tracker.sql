-- 追踪的动漫：把动漫与 kisssub 上的详情页绑定
CREATE TABLE IF NOT EXISTS tracked_anime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL UNIQUE,
    search_keyword TEXT NOT NULL,
    kisssub_url TEXT,
    last_episode_title TEXT,
    last_episode_group TEXT,
    last_checked_at DATETIME,
    has_update INTEGER NOT NULL DEFAULT 0,
    last_seen_episode_title TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

-- 每次检查发现的更新记录
CREATE TABLE IF NOT EXISTS update_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tracked_anime_id INTEGER NOT NULL,
    episode_title TEXT NOT NULL,
    subtitle_group TEXT,
    found_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tracked_anime_id) REFERENCES tracked_anime(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tracked_anime_id ON tracked_anime(anime_id);
CREATE INDEX IF NOT EXISTS idx_update_history_tracked ON update_history(tracked_anime_id, found_at DESC);
