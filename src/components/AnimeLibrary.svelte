<script lang="ts">
  import { onMount } from "svelte";
  import type { Anime } from "../types/anime";
  import { loadCoverImage } from "../utils/image";

  export let animeList: Anime[];
  export let onSelect: (anime: Anime) => void;

  const gradients = [
    "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
    "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
    "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
    "linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)",
    "linear-gradient(135deg, #fa709a 0%, #fee140 100%)",
    "linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)",
    "linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%)",
    "linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)",
  ];

  interface AnimeGroup {
    title: string;
    versions: Anime[];
    seasonCount: number;
    totalEpisodes: number;
    watchedEpisodes: number;
    primaryAnime: Anime;
  }

  let coverCache: Record<number, string | null> = {};
  const loadingCovers = new Set<number>();

  function getGradient(id: number): string {
    return gradients[id % gradients.length];
  }

  function handleKey(e: KeyboardEvent, anime: Anime) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect(anime);
    }
  }

  async function loadCover(anime: Anime) {
    if (anime.id in coverCache || loadingCovers.has(anime.id)) {
      return;
    }

    const coverPaths: string[] = [];

    if (anime.cover_image) {
      coverPaths.push(anime.cover_image);
    }

    if (anime.directory_path) {
      const dir = anime.directory_path;
      coverPaths.push(
        `${dir}\\cover.jpg`,
        `${dir}\\cover.png`,
        `${dir}\\cover.webp`,
        `${dir}\\cover.gif`,
        `${dir}\\poster.jpg`,
        `${dir}\\poster.png`,
        `${dir}\\folder.jpg`,
        `${dir}\\folder.png`
      );
    }

    if (coverPaths.length === 0) {
      return;
    }

    loadingCovers.add(anime.id);
    try {
      let dataUrl: string | null = null;
      for (const path of coverPaths) {
        dataUrl = await loadCoverImage(path);
        if (dataUrl) break;
      }
      if (dataUrl) {
        coverCache[anime.id] = dataUrl;
        coverCache = coverCache;
      }
    } catch {
      console.error(`Error loading cover for ${anime.title}`);
    } finally {
      loadingCovers.delete(anime.id);
    }
  }

  let groups: AnimeGroup[] = [];

  $: {
    const map = new Map<string, Anime[]>();
    for (const a of animeList) {
      const list = map.get(a.title) || [];
      list.push(a);
      map.set(a.title, list);
    }

    const result: AnimeGroup[] = [];
    for (const [title, versions] of map) {
      versions.sort((a, b) => {
        if (a.is_movie !== b.is_movie) return a.is_movie ? 1 : -1;
        if (a.season !== b.season) return a.season - b.season;
        return (a.subtitle_group || "").localeCompare(b.subtitle_group || "");
      });

      const seasons = new Set(versions.map(v => v.season));
      const totalEpisodes = versions.reduce((s, v) => s + v.total_episodes, 0);
      const watchedEpisodes = versions.reduce((s, v) => s + v.watched_episodes, 0);
      const primaryAnime = versions.find(v => !v.is_movie) || versions[0];

      result.push({
        title,
        versions,
        seasonCount: seasons.size,
        totalEpisodes,
        watchedEpisodes,
        primaryAnime,
      });
    }

    result.sort((a, b) => a.title.localeCompare(b.title));
    groups = result;
  }

  onMount(() => {
    animeList.forEach(loadCover);
  });

  $: {
    animeList.forEach(loadCover);
  }
</script>

{#if groups.length === 0}
  <div class="empty-state">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="2" y="3" width="20" height="14" rx="2"/>
        <path d="M8 21h8M12 17v4"/>
      </svg>
    </div>
    <h2>动漫库为空</h2>
    <p>点击右上角"扫描库"开始扫描</p>
  </div>
{:else}
  <div class="anime-grid">
    {#each groups as g (g.title)}
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class="anime-card"
        role="button"
        tabindex="0"
        on:click={() => onSelect(g.primaryAnime)}
        on:keydown={(e) => handleKey(e, g.primaryAnime)}
      >
        <div class="card-cover">
          {#if coverCache[g.primaryAnime.id]}
            <img src={coverCache[g.primaryAnime.id]} alt={g.title} loading="lazy" />
          {:else}
            <div class="cover-gradient" style="background: {getGradient(g.primaryAnime.id)}">
              <svg class="cover-icon" viewBox="0 0 48 48" width="40" height="40" fill="none" stroke="rgba(255,255,255,0.6)" stroke-width="1.5">
                <circle cx="24" cy="24" r="18"/>
                <path d="M18 20v8M24 16v16M30 20v8"/>
              </svg>
            </div>
          {/if}
          {#if g.seasonCount > 1}
            <div class="season-badge">
              <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>
              {g.seasonCount} 季
            </div>
          {/if}
        </div>
        <div class="card-body">
          <div class="title-wrap">
            <p 
              class="card-title copyable" 
              on:click={(e) => { e.stopPropagation(); navigator.clipboard.writeText(g.title); }}
            >{g.title}</p>
          </div>
          <div class="card-meta">
            <span class="meta-ep">{g.totalEpisodes} 集</span>
            {#if g.versions.length > 1 && g.seasonCount <= 1}
              <span class="meta-sub">{g.versions.length} 个版本</span>
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 60vh;
    gap: var(--space-3);
    color: var(--text-tertiary);
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--accent-200), var(--accent-100));
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-400);
    margin-bottom: var(--space-2);
  }

  .empty-state h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .empty-state p {
    font-size: 0.85rem;
  }

  .anime-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-6);
  }

  .anime-card {
    display: flex;
    flex-direction: column;
    background: var(--card-bg);
    border-radius: var(--radius-lg);
    overflow: hidden;
    cursor: pointer;
    transition: all var(--transition-base);
    box-shadow: var(--card-shadow);
    border: 1px solid var(--border);
    position: relative;
  }

  .anime-card:hover {
    transform: translateY(var(--hover-lift));
    box-shadow: var(--card-shadow-hover);
    border-color: var(--border-strong);
  }

  .anime-card:hover .card-cover img {
    transform: scale(1.05);
  }

  .card-cover {
    position: relative;
    width: 100%;
    aspect-ratio: 3 / 4;
    overflow: hidden;
    flex-shrink: 0;
  }

  .card-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.4s ease;
  }

  .cover-gradient {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover-icon {
    opacity: 0.8;
  }

  .season-badge {
    position: absolute;
    top: 10px;
    left: 10px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(8px);
    color: white;
    border-radius: var(--radius-pill);
    font-size: 0.72rem;
    font-weight: 600;
    pointer-events: none;
  }

  .card-body {
    padding: var(--space-3) var(--space-4) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-height: 0;
    flex: 1;
  }

  .title-wrap {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    word-break: break-word;
    overflow-wrap: break-word;
  }

  .card-title {
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.45;
    margin: 0;
  }

  .card-title.copyable:hover,
  .card-title.copyable:active {
    color: var(--accent-500);
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.75rem;
    margin-top: auto;
    min-width: 0;
  }

  .meta-ep {
    color: var(--text-tertiary);
    font-weight: 400;
    flex-shrink: 0;
  }

  .meta-sub {
    color: var(--accent-500);
    font-weight: 500;
    margin-left: auto;
  }

  /* 暗色模式适配 */
  :global([data-theme="dark"]) .empty-icon {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.25), rgba(var(--accent-rgb), 0.15));
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .empty-state h2 {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .empty-state p {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .anime-card {
    background: var(--card-bg);
    border-color: var(--border);
    box-shadow: var(--card-shadow);
  }

  :global([data-theme="dark"]) .anime-card:hover {
    border-color: var(--border-strong);
    box-shadow: var(--card-shadow-hover);
  }

  :global([data-theme="dark"]) .card-title {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .meta-ep {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .meta-sub {
    color: var(--accent-400);
  }

  :global([data-theme="dark"]) .season-badge {
    background: rgba(0, 0, 0, 0.6);
  }
</style>
