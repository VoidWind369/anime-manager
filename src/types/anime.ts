export interface Episode {
  id: number;
  anime_id: number;
  title: string;
  file_path: string;
  episode_number: number;
  duration: number | null;
  watched: boolean;
  watch_progress: number;
}

export interface Anime {
  id: number;
  title: string;
  original_title: string | null;
  subtitle: string | null;
  season: number;
  subtitle_group: string | null;
  directory_path: string;
  cover_image: string | null;
  description: string | null;
  total_episodes: number;
  watched_episodes: number;
  is_movie: boolean;
  added_at: string;
  episodes?: Episode[];
}

export interface ScanResult {
  total: number;
  added: number;
  updated: number;
  removed: number;
}
