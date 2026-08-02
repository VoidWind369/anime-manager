<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "../utils/tauri-adapter";
  import type { Anime, Episode } from "../types/anime";
  import { loadCoverImage } from "../utils/image";

  export let anime: Anime;
  export let onRefresh: () => void;

  let allVersions: Anime[] = [];
  let allEpisodes: Map<number, Episode[]> = new Map();
  let isLoading = true;
  let originalName = "";
  let coverDataUrl: string | null = null;

  let showModal = false;
  let modalTitle = "";
  let modalMessage = "";

  function showMessage(title: string, message: string) {
    modalTitle = title;
    modalMessage = message;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
  }

  interface SeasonGroup {
    season: number;
    isMovie: boolean;
    totalEpisodes: number;
    watchedEpisodes: number;
    versions: VersionGroup[];
  }

  interface VersionGroup {
    anime: Anime;
    episodes: Episode[];
  }

  let seasonGroups: SeasonGroup[] = [];
  let activeSeason: number = 1;
  let activeVersionIndex: number = 0;

  $: totalEpisodesAll = seasonGroups.reduce((s, g) => s + g.totalEpisodes, 0);
  $: watchedEpisodesAll = seasonGroups.reduce((s, g) => s + g.watchedEpisodes, 0);

  $: activeGroup = seasonGroups.find(g => g.season === activeSeason) || null;
  $: activeVersion = activeGroup?.versions[activeVersionIndex] || null;

  $: {
    if (activeGroup && activeVersionIndex >= activeGroup.versions.length) {
      activeVersionIndex = 0;
    }
  }

  $: {
    const seasonMap = new Map<number, VersionGroup[]>();
    for (const v of allVersions) {
      const eps = allEpisodes.get(v.id) || [];
      const list = seasonMap.get(v.season) || [];
      list.push({ anime: v, episodes: eps });
      seasonMap.set(v.season, list);
    }

    const result: SeasonGroup[] = [];
    for (const [season, versions] of seasonMap) {
      versions.sort((a, b) =>
        (a.anime.subtitle_group || "").localeCompare(b.anime.subtitle_group || "")
      );
      const isMovie = versions.some((v) => v.anime.is_movie);
      const totalEpisodes = versions.reduce((s, v) => s + v.episodes.length, 0);
      const watchedEpisodes = versions.reduce(
        (s, v) => s + v.episodes.filter((e) => e.watched).length,
        0
      );
      result.push({ season, isMovie, totalEpisodes, watchedEpisodes, versions });
    }
    result.sort((a, b) => a.season - b.season);
    seasonGroups = result;
  }

  async function loadCover() {
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
      coverDataUrl = null;
      return;
    }

    try {
      let dataUrl: string | null = null;
      for (const path of coverPaths) {
        dataUrl = await loadCoverImage(path);
        if (dataUrl) break;
      }
      coverDataUrl = dataUrl;
    } catch {
      coverDataUrl = null;
    }
  }

  onMount(async () => {
    await loadAllVersions();
    await loadOriginalName();
    await loadCover();
    activeSeason = anime.season;
  });

  $: {
    anime;
    loadCover();
  }

  async function loadAllVersions() {
    isLoading = true;
    try {
      allVersions = await invoke<Anime[]>("get_anime_by_title", { title: anime.title });
    } catch {
      allVersions = [anime];
    }
    const epMap = new Map<number, Episode[]>();
    for (const v of allVersions) {
      try {
        const eps = await invoke<Episode[]>("get_anime_episodes", { animeId: v.id });
        epMap.set(v.id, eps);
      } catch {
        epMap.set(v.id, []);
      }
    }
    allEpisodes = epMap;
    isLoading = false;
  }

  async function loadOriginalName() {
    try { originalName = await invoke<string>("get_original_name", { animeId: anime.id }); }
    catch (_) { originalName = ""; }
  }

  async function playEpisode(episode: Episode) {
    try { await invoke("play_episode", { episodeId: episode.id }); }
    catch (e) { showMessage("播放失败", `${e}`); }
  }

  async function toggleWatched(episode: Episode) {
    try {
      await invoke("toggle_episode_watched", { episodeId: episode.id });
      episode.watched = !episode.watched;
      onRefresh();
    } catch (_) {}
  }

  async function openDirectory(path?: string) {
    try { await invoke("open_directory", { path: path || anime.directory_path }); } catch (_) {}
  }

  function formatDate(s: string | null): string {
    if (!s) return "";
    const d = new Date(s);
    if (isNaN(d.getTime())) return s;
    return d.toLocaleString("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" });
  }

  function seasonLabel(season: number, isMovie: boolean): string {
    if (isMovie) return "剧场版";
    return `第${season}季`;
  }
</script>

<div class="detail-wrap">
  <!-- 头部 -->
  <div class="detail-hero">
    <div class="hero-cover">
      {#if coverDataUrl}
        <img src={coverDataUrl} alt={anime.title} />
      {:else}
        <div class="cover-fallback"></div>
      {/if}
    </div>
    <div class="hero-info">
      <h1 
      class="hero-title copyable" 
      on:click={() => navigator.clipboard.writeText(anime.title)}
    >{anime.title}</h1>
      {#if originalName}
        <p class="hero-original">原名: {originalName}</p>
      {/if}
      <div class="hero-tags">
        <span class="tag-primary">
          {#if activeGroup?.isMovie}
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="16" rx="2"/><path d="M8 2v16M16 2v16"/><circle cx="6" cy="10" r="1"/><circle cx="18" cy="10" r="1"/><circle cx="6" cy="14" r="1"/><circle cx="18" cy="14" r="1"/></svg>
          {:else}
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/><text x="12" y="11" text-anchor="middle" font-size="8" font-weight="700" fill="currentColor">{activeSeason}</text></svg>
          {/if}
          {activeGroup ? seasonLabel(activeSeason, activeGroup.isMovie) : seasonLabel(anime.season, anime.is_movie)}
        </span>
        {#if activeVersion?.anime.subtitle_group}
          {#each activeVersion.anime.subtitle_group.split('&') as group}
            <span class="tag-sub">{group.trim()}</span>
          {/each}
        {:else if anime.subtitle_group}
          {#each anime.subtitle_group.split('&') as group}
            <span class="tag-sub">{group.trim()}</span>
          {/each}
        {/if}
        {#if allVersions.length > 1}
          <span class="tag-version">{allVersions.length} 个版本 / {seasonGroups.length} 季</span>
        {/if}
      </div>
      {#if anime.description}
        <p class="hero-desc">{anime.description}</p>
      {/if}
    </div>
  </div>

  <!-- 剧集列表（季选项卡 + 字幕组分页） -->
  <section class="ep-section">
    <div class="ep-heading-row">
      <h2 class="ep-heading">剧集列表</h2>
      <span class="ep-count">
        共 {totalEpisodesAll} 集 /
        已看 {watchedEpisodesAll} 集
      </span>
    </div>

    {#if isLoading}
      <div class="ep-loading">加载中...</div>
    {:else if seasonGroups.length === 0}
      <div class="ep-empty">
        <p>未找到视频文件</p>
        <span class="ep-hint">mp4 / mkv / avi / mov / wmv / flv / webm / rmvb / m4v</span>
      </div>
    {:else}
      <!-- 季选项卡 -->
      {#if seasonGroups.length > 1}
        <div class="season-tabs">
          {#each seasonGroups as sg (sg.season)}
            <button
              class="season-tab"
              class:active={sg.season === activeSeason}
              on:click={() => activeSeason = sg.season}
            >
              {seasonLabel(sg.season, sg.isMovie)}
              <span class="tab-ep-count">{sg.totalEpisodes}</span>
            </button>
          {/each}
        </div>
      {/if}

      {#if activeGroup}
        <!-- 版本选项卡 -->
        {#if activeGroup.versions.length > 1}
          <div class="version-tabs">
            {#each activeGroup.versions as vg, i (vg.anime.id)}
              <button
                class="version-tab"
                class:active={i === activeVersionIndex}
                on:click={() => activeVersionIndex = i}
              >
                {#if vg.anime.subtitle}
                  <span class="vtab-name">{vg.anime.subtitle}</span>
                {/if}
                {#if vg.anime.subtitle_group}
                  <span class="vtab-group">{vg.anime.subtitle_group}</span>
                {:else}
                  <span class="vtab-group unknown">未知字幕组</span>
                {/if}
                <span class="vtab-count">{vg.episodes.length}</span>
              </button>
            {/each}
          </div>
        {/if}

        {#if activeVersion}
          <div class="version-group">
            <div class="version-header">
              <div class="version-info">
                {#if activeVersion.anime.subtitle}
                  <span class="version-name">{activeVersion.anime.subtitle}</span>
                {/if}
                {#if activeVersion.anime.subtitle_group}
                  {#each activeVersion.anime.subtitle_group.split('&') as g}
                    <span class="version-sub-tag">{g.trim()}</span>
                  {/each}
                {:else}
                  <span class="version-sub-tag unknown">未知字幕组</span>
                {/if}
                <span class="version-ep-count">{activeVersion.episodes.length} 集</span>
              </div>
              <button class="version-dir-btn" on:click={() => openDirectory(activeVersion.anime.directory_path)} title="打开此版本目录">
                <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                目录
              </button>
            </div>

            <div class="ep-list">
              {#if activeVersion.episodes.length === 0}
                <div class="ep-row-empty">暂无剧集文件</div>
              {:else}
                {#each activeVersion.episodes as ep (ep.id)}
                  <div class="ep-row" class:watched={ep.watched}>
                    <span class="ep-num">{String(ep.episode_number).padStart(2, "0")}</span>
                    <span 
                      class="ep-name copyable" 
                      on:click={(e) => { e.stopPropagation(); navigator.clipboard.writeText(ep.title); }}
                    >{ep.title}</span>
                    <div class="ep-actions">
                      <button class="play-btn" on:click={() => playEpisode(ep)} aria-label="播放">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
                      </button>
                      <button
                        class="watch-btn"
                        class:done={ep.watched}
                        on:click={() => toggleWatched(ep)}
                        aria-label={ep.watched ? "标记未看" : "标记已看"}
                      >
                        {#if ep.watched}
                          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                        {:else}
                          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/></svg>
                        {/if}
                      </button>
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/if}
      {/if}
    {/if}
  </section>
</div>

<style>
  .detail-wrap { max-width: 960px; margin: 0 auto; }

  .detail-hero {
    display: flex;
    gap: var(--space-8);
    margin-bottom: var(--space-10);
    padding-bottom: var(--space-8);
    border-bottom: 1px solid var(--border);
  }

  .hero-cover { flex-shrink: 0; }

  .hero-cover img {
    width: 180px;
    aspect-ratio: 3 / 4;
    object-fit: cover;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }

  .cover-fallback {
    width: 180px;
    aspect-ratio: 3 / 4;
    border-radius: var(--radius-lg);
    background: linear-gradient(135deg, var(--accent-200), var(--accent-100), var(--accent-50));
  }

  .hero-info { flex: 1; padding-top: var(--space-2); }

  .hero-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: var(--space-1);
  }

  .hero-title.copyable:hover,
  .hero-title.copyable:active {
    color: var(--accent-500);
  }

  .hero-original {
    font-size: 0.82rem;
    color: var(--text-secondary);
    margin-bottom: var(--space-4);
  }

  .hero-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }

  .tag-primary {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    color: white;
    padding: 4px 10px;
    border-radius: var(--radius-pill);
    font-size: 0.78rem;
    font-weight: 600;
    box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.2);
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .tag-plain {
    color: var(--text-secondary);
    font-size: 0.82rem;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .tag-sub {
    color: var(--accent-500);
    font-size: 0.82rem;
    font-weight: 500;
    padding: 3px 12px;
    background: rgba(255, 255, 255, 0.55);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border-radius: var(--radius-pill);
    border: 1px solid rgba(255, 85, 149, 0.12);
  }

  .tag-version {
    color: var(--text-secondary);
    font-size: 0.75rem;
    padding: 3px 8px;
    background: var(--surface-dim);
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
  }

  .hero-desc {
    color: var(--text-secondary);
    font-size: 0.88rem;
    line-height: 1.7;
    margin-bottom: var(--space-5);
  }

  /* 剧集列表 */
  .ep-section {
    padding: var(--space-6);
  }

  .ep-heading-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .ep-heading {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .ep-count {
    font-size: 0.75rem;
    color: var(--text-secondary);
    padding: 3px 10px;
    background: rgba(255, 255, 255, 0.55);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .ep-loading, .ep-empty {
    text-align: center;
    padding: var(--space-10);
    color: var(--text-tertiary);
  }

  .ep-hint {
    display: block;
    font-size: 0.78rem;
    margin-top: var(--space-2);
    color: var(--text-tertiary);
    opacity: 0.7;
  }

  /* 季选项卡 */
  .season-tabs {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }

  .season-tab {
    flex: 0 0 auto;
    padding: 8px 18px;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }

  .season-tab:hover {
    color: var(--text-primary);
    background: var(--surface);
  }

  .season-tab.active {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    color: white;
    box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.3);
  }

  .season-tab.active:hover {
    color: white;
  }

  .tab-ep-count {
    font-size: 0.72rem;
    font-weight: 500;
    opacity: 0.75;
    padding: 1px 7px;
    background: rgba(0,0,0,0.15);
    border-radius: var(--radius-pill);
  }

  .season-tab:not(.active) .tab-ep-count {
    background: var(--border);
    color: var(--text-tertiary);
    opacity: 1;
  }

  /* 版本选项卡 */
  .version-tabs {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .version-tab {
    flex: 0 0 auto;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }

  .version-tab:hover {
    color: var(--text-primary);
    background: var(--surface-dim);
  }

  .version-tab.active {
    background: var(--accent-50);
    color: var(--accent-600);
    font-weight: 600;
    border: 1px solid var(--accent-200);
    padding: 5px 13px;
  }

  .vtab-name {
    color: var(--text-primary);
    font-weight: 600;
  }

  .version-tab.active .vtab-name {
    color: var(--accent-700);
  }

  .vtab-group {
    color: var(--accent-500);
    font-weight: 500;
  }

  .version-tab:not(.active) .vtab-group {
    color: var(--text-tertiary);
    font-weight: 400;
  }

  .vtab-group.unknown {
    color: var(--text-tertiary);
  }

  .vtab-count {
    font-size: 0.68rem;
    font-weight: 500;
    padding: 1px 6px;
    background: var(--border);
    color: var(--text-tertiary);
    border-radius: var(--radius-pill);
  }

  .version-tab.active .vtab-count {
    background: var(--accent-200);
    color: var(--accent-700);
  }

  .version-group {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .version-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-5);
    background: var(--card-bg-dim);
    border-bottom: 1px solid var(--border);
  }

  .version-info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .version-name {
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .version-sub-tag {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--accent-600);
    background: var(--accent-50);
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--accent-100);
  }

  .version-sub-tag.unknown {
    color: var(--text-tertiary);
    background: transparent;
    border-color: var(--border);
  }

  .version-ep-count {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    margin-left: var(--space-2);
  }

  .version-dir-btn {
    font-size: 0.75rem;
    padding: 6px 12px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--text-secondary);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: none;
  }

  .version-dir-btn:hover {
    background: var(--accent-50);
    color: var(--accent-600);
    border-color: var(--accent-200);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.15);
  }

  .ep-list {
    display: flex;
    flex-direction: column;
    padding: var(--space-2) 0;
  }

  .ep-row-empty {
    padding: var(--space-4) var(--space-5);
    color: var(--text-tertiary);
    font-size: 0.82rem;
    text-align: center;
  }

  .ep-row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-5);
    transition: all 0.2s ease;
  }

  .ep-row:hover {
    background: var(--card-bg-hover);
  }

  .ep-row.watched { opacity: 0.45; }

  .ep-num {
    min-width: 28px;
    text-align: center;
    font-size: 0.85rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    background: linear-gradient(135deg, var(--accent-600), var(--accent-400));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .ep-name {
    flex: 1;
    min-width: 0;
    font-size: 0.88rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .ep-name.copyable:hover,
  .ep-name.copyable:active {
    color: var(--accent-500);
  }

  .ep-actions { display: flex; gap: var(--space-2); }

  .play-btn, .watch-btn {
    width: 30px; height: 30px; padding: 0;
    border-radius: var(--radius-sm);
  }

  .play-btn {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    color: white;
    box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.2);
  }

  .watch-btn {
    background: var(--accent-50);
    color: var(--text-tertiary);
    border: 1px solid var(--border);
    box-shadow: none;
  }

  .watch-btn:hover { color: var(--accent-500); border-color: var(--border-strong); }

  .watch-btn.done {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    color: white;
    border: none;
    box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.2);
  }

  

  /* ========== 暗色模式适配 ========== */
  :global([data-theme="dark"]) .detail-hero {
    border-bottom-color: var(--border);
  }

  :global([data-theme="dark"]) .cover-fallback {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.3), rgba(var(--accent-rgb), 0.15), rgba(var(--accent-rgb), 0.4));
  }

  :global([data-theme="dark"]) .hero-title {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .hero-original {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .tag-plain {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .tag-sub {
    color: var(--accent-200);
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 123, 171, 0.12);
  }

  :global([data-theme="dark"]) .tag-version {
    background: var(--surface-dim);
    border-color: var(--border);
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .hero-desc {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .ep-count {
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.06);
  }

  /* 季选项卡暗色 */
  :global([data-theme="dark"]) .season-tab {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .season-tab:hover {
    background: var(--surface);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .season-tab.active {
    color: white;
  }

  :global([data-theme="dark"]) .season-tab.active:hover {
    color: white;
  }

  :global([data-theme="dark"]) .season-tab:not(.active) .tab-ep-count {
    background: var(--border);
    color: var(--text-tertiary);
  }

  /* 版本选项卡暗色 */

  :global([data-theme="dark"]) .version-tab {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .version-tab:hover {
    background: var(--surface-dim);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .version-tab.active {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent-300);
    border-color: rgba(var(--accent-rgb), 0.3);
  }

  :global([data-theme="dark"]) .version-tab.active .vtab-name {
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .version-tab.active .vtab-group {
    color: var(--accent-400);
  }

  :global([data-theme="dark"]) .vtab-group {
    color: var(--accent-400);
  }

  :global([data-theme="dark"]) .version-tab:not(.active) .vtab-group {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .vtab-count {
    background: var(--border);
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .version-tab.active .vtab-count {
    background: rgba(var(--accent-rgb), 0.25);
    color: var(--accent-300);
  }

  /* 版本组暗色 */
  :global([data-theme="dark"]) .version-group {
    background: var(--card-bg);
    border-color: var(--border);
  }

  :global([data-theme="dark"]) .version-header {
    background: var(--card-bg-dim);
    border-color: var(--border);
  }

  :global([data-theme="dark"]) .version-name {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .version-sub-tag {
    color: var(--accent-400);
    background: rgba(var(--accent-rgb), 0.12);
    border-color: rgba(var(--accent-rgb), 0.2);
  }

  :global([data-theme="dark"]) .version-sub-tag.unknown {
    color: var(--text-tertiary);
    background: transparent;
    border-color: var(--border);
  }

  :global([data-theme="dark"]) .version-ep-count {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .version-dir-btn {
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(var(--accent-rgb), 0.15);
    box-shadow: none;
  }

  :global([data-theme="dark"]) .version-dir-btn:hover {
    color: var(--accent-300);
    background: rgba(var(--accent-rgb), 0.15);
    border-color: rgba(var(--accent-rgb), 0.35);
    box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.15);
  }

  /* 剧集列表暗色 */
  :global([data-theme="dark"]) .ep-heading {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .ep-count {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .ep-loading,
  :global([data-theme="dark"]) .ep-empty {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .ep-hint {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .ep-row:hover {
    background: var(--card-bg-hover);
  }

  :global([data-theme="dark"]) .ep-name {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .play-btn {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.95), rgba(var(--accent-rgb), 0.9));
    color: white;
    box-shadow: 0 2px 10px rgba(var(--accent-rgb), 0.35);
  }

  :global([data-theme="dark"]) .play-btn:hover {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.98), rgba(var(--accent-rgb), 0.95));
    box-shadow: 0 4px 16px rgba(var(--accent-rgb), 0.5);
  }

  :global([data-theme="dark"]) .watch-btn {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(var(--accent-rgb), 0.15);
    color: var(--text-tertiary);
    box-shadow: none;
  }

  :global([data-theme="dark"]) .watch-btn:hover {
    color: var(--accent-300);
    border-color: rgba(var(--accent-rgb), 0.35);
    background: rgba(var(--accent-rgb), 0.15);
  }

  :global([data-theme="dark"]) .watch-btn.done {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.95), rgba(var(--accent-rgb), 0.9));
    color: white;
    box-shadow: 0 2px 10px rgba(var(--accent-rgb), 0.35);
  }

  :global([data-theme="dark"]) .ep-row-empty {
    color: var(--text-tertiary);
  }

  
</style>
