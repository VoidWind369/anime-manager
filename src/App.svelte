<script lang="ts">
  import { onMount, afterUpdate } from "svelte";
  import { invoke } from "./utils/tauri-adapter";
  import { _ } from "svelte-i18n";
  import "./i18n/index";
  import { loadAllPlugins, panels, toolbarButtons, resolveModal, showToast as pluginToast } from "./plugins/index";
  import TitleBar from "./components/TitleBar.svelte";
  import AnimeLibrary from "./components/AnimeLibrary.svelte";
  import AnimeDetail from "./components/AnimeDetail.svelte";
  import Settings from "./components/Settings.svelte";
  import type { Anime, ScanResult } from "./types/anime";

  let currentView: "library" | "detail" | "settings" = "library";
  let selectedAnime: Anime | null = null;
  let animeList: Anime[] = [];
  let searchQuery = "";
  let isScanning = false;
  let libraryPath = "E:\\动漫";
  let isDark = false;
  let colorScheme = "sakura";
  let radiusScale = 1;
  let bgStyle = "pure";
  let glassMode = "solid";
  let shadowStyle = "standard";
  let motionLevel = "standard";
  let customBgUrl = "";
  let bgOpacity = 0.5;
  let cardOpacity = 1;
  let titlebarOpacity = 0.8;

  let pluginToolbarVersion = 0;
  $: currentPluginToolbarButtons = pluginToolbarVersion >= 0 ? Array.from(toolbarButtons.values()) : [];

  let showModal = false;
  let modalTitle = "";
  let modalMessage = "";
  let modalContentHtml: HTMLElement | null = null;
  let showConfirm = false;
  let confirmCallback: (() => void) | null = null;

  function showMessage(title: string, message: string) {
    modalTitle = title;
    modalMessage = message;
    modalContentHtml = null;
    showConfirm = false;
    confirmCallback = null;
    showModal = true;
  }

  function showConfirmDialog(title: string, message: string, callback: () => void) {
    modalTitle = title;
    modalMessage = message;
    showConfirm = true;
    confirmCallback = callback;
    showModal = true;
  }

  let modalBodyEl: HTMLDivElement;

  afterUpdate(() => {
    if (modalBodyEl && modalContentHtml) {
      modalBodyEl.innerHTML = '';
      modalBodyEl.appendChild(modalContentHtml);
    }
  });

  function closeModal() {
    showModal = false;
    modalContentHtml = null;
    confirmCallback = null;
  }

  function handleConfirm() {
    if (confirmCallback) {
      confirmCallback();
    }
    closeModal();
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = text;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
  }

  $: filteredAnime = animeList.filter((a) =>
    a.title.toLowerCase().includes(searchQuery.toLowerCase())
  );

  onMount(async () => {
    colorScheme = loadColorScheme();
    applyColorScheme(colorScheme);
    radiusScale = parseNumberSetting("radius-scale", 1);
    applyRadiusScale(radiusScale);
    bgStyle = loadSetting("bg-style", "pure");
    applyBgStyle(bgStyle);
    glassMode = loadSetting("glass-mode", "solid");
    applyGlassMode(glassMode);
    shadowStyle = loadSetting("shadow-style", "standard");
    applyShadowStyle(shadowStyle);
    motionLevel = loadSetting("motion-level", "standard");
    applyMotionLevel(motionLevel);
    customBgUrl = loadSetting("custom-bg-url", "");
    applyCustomBgUrl(customBgUrl);
    bgOpacity = parseNumberSetting("bg-opacity", 0.5);
    applyBgOpacity(bgOpacity);
    cardOpacity = parseNumberSetting("card-opacity", 1);
    applyCardOpacity(cardOpacity);
    titlebarOpacity = parseNumberSetting("titlebar-opacity", 0.8);
    applyTitlebarOpacity(titlebarOpacity);
    applyTheme(loadTheme());
    await loadSettings();
    await loadAnimeList();

    setupPluginBridge();
    await loadAllPlugins();

    document.addEventListener("contextmenu", (e) => e.preventDefault());
  });

  function setupPluginBridge() {
    const w = window as any;
    w.__pluginAnimeList = () => animeList;
    w.__pluginSettings = () => ({
      libraryPath, isDark, colorScheme, bgStyle, glassMode, shadowStyle, motionLevel,
    });
    w.__pluginNavigate = (view: string, data?: any) => {
      if (view === "library") { currentView = "library"; }
      else if (view === "detail" && data) { selectedAnime = data; currentView = "detail"; }
      else if (view === "settings") { currentView = "settings"; }
    };
    w.__pluginShowModal = (options: any) => {
      modalTitle = options.title;
      if (options.content instanceof HTMLElement) {
        modalContentHtml = options.content;
        modalMessage = '';
      } else {
        modalMessage = typeof options.content === 'string' ? options.content : '';
        modalContentHtml = null;
      }
      showConfirm = false;
      confirmCallback = null;
      showModal = true;
    };
    w.__pluginShowToast = (options: any) => {
      showMessage(options.message, options.type ?? 'info');
    };
    w.__pluginToolbarChanged = () => {
      pluginToolbarVersion++;
    };
    w.__pluginPanelsChanged = () => {
      pluginToolbarVersion++;
    };
  }

  function loadTheme(): boolean {
    try {
      const saved = localStorage.getItem("theme");
      if (saved) return saved === "dark";
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    } catch (_) {
      return false;
    }
  }

  function loadColorScheme(): string {
    try {
      return localStorage.getItem("color-scheme") || "sakura";
    } catch (_) {
      return "sakura";
    }
  }

  function applyTheme(dark: boolean) {
    isDark = dark;
    if (dark) {
      document.documentElement.setAttribute("data-theme", "dark");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
    try {
      localStorage.setItem("theme", dark ? "dark" : "light");
    } catch (_) {}
  }

  function applyColorScheme(scheme: string) {
    colorScheme = scheme;
    document.documentElement.setAttribute("data-color-scheme", scheme);
    try {
      localStorage.setItem("color-scheme", scheme);
    } catch (_) {}
  }

  function loadSetting(key: string, fallback: string): string {
    try {
      return localStorage.getItem(key) || fallback;
    } catch (_) {
      return fallback;
    }
  }

  function parseNumberSetting(key: string, fallback: number): number {
    const raw = localStorage.getItem(key);
    if (raw === null) return fallback;
    const n = parseFloat(raw);
    return isNaN(n) ? fallback : n;
  }

  function applyAttr(key: string, attr: string, value: string) {
    document.documentElement.setAttribute(attr, value);
    try {
      localStorage.setItem(key, value);
    } catch (_) {}
  }

  function applyRadiusScale(v: number) {
    radiusScale = Math.max(0, Math.min(1, v));
    try { localStorage.setItem("radius-scale", String(radiusScale)); } catch (_) {}
    const p = radiusScale;
    const el = document.documentElement;
    el.style.setProperty("--radius-sm", `${Math.round(24 * p)}px`);
    el.style.setProperty("--radius-md", `${Math.round(30 * p)}px`);
    el.style.setProperty("--radius-lg", `${Math.round(40 * p)}px`);
    el.style.setProperty("--radius-pill", p <= 0 ? "0px" : "999px");
  }
  function applyCardOpacity(v: number) {
    cardOpacity = Math.max(0.2, Math.min(1, v));
    document.documentElement.style.setProperty("--card-opacity", String(cardOpacity));
    try { localStorage.setItem("card-opacity", String(cardOpacity)); } catch (_) {}
  }
  function applyBgOpacity(v: number) {
    bgOpacity = Math.max(0.1, Math.min(1, v));
    document.documentElement.style.setProperty("--bg-image-opacity", String(bgOpacity));
    try { localStorage.setItem("bg-opacity", String(bgOpacity)); } catch (_) {}
  }
  function applyTitlebarOpacity(v: number) {
    titlebarOpacity = Math.max(0, Math.min(1, v));
    document.documentElement.style.setProperty("--titlebar-opacity", String(titlebarOpacity));
    try { localStorage.setItem("titlebar-opacity", String(titlebarOpacity)); } catch (_) {}
  }
  function applyBgStyle(v: string, url?: string) {
    bgStyle = v;
    applyAttr("bg-style", "data-bg-style", v);
    if (v === "pure" || v === "gradient") {
      customBgUrl = "";
      try { localStorage.setItem("custom-bg-url", ""); } catch (_) {}
      document.documentElement.style.setProperty("--custom-bg-image", "none");
    } else if (v === "custom" && url !== undefined) {
      applyCustomBgUrl(url);
    }
  }
  function applyGlassMode(v: string) { glassMode = v; applyAttr("glass-mode", "data-glass", v); }
  function applyShadowStyle(v: string) { shadowStyle = v; applyAttr("shadow-style", "data-shadow", v); }
  function applyMotionLevel(v: string) { motionLevel = v; applyAttr("motion-level", "data-motion", v); }
  function applyCustomBgUrl(url: string) {
    customBgUrl = url;
    document.documentElement.style.setProperty("--custom-bg-image", url ? `url("${url}")` : "none");
    try {
      localStorage.setItem("custom-bg-url", url);
    } catch (_) {}
  }

  function toggleTheme() {
    applyTheme(!isDark);
  }

  function changeColorScheme(scheme: string) {
    applyColorScheme(scheme);
  }

  async function loadSettings() {
    try {
      const saved = await invoke<string>("get_library_path");
      if (saved) libraryPath = saved;
    } catch (_) {}
  }

  async function loadAnimeList() {
    try {
      animeList = await invoke<Anime[]>("get_anime_list");
      console.log(`Loaded ${animeList.length} anime`);
    } catch (e) {
      console.error('Failed to load anime list:', e);
      animeList = [];
    }
  }

  async function scanLibrary() {
    if (isScanning) return;
    isScanning = true;
    try {
      const result = await invoke<ScanResult>("scan_library", { path: libraryPath });
      await loadAnimeList();
      let msg = $_('scan.result', { values: { total: result.total, added: result.added, updated: result.updated } });
      if (result.removed > 0) msg += $_('scan.cleaned', { values: { removed: result.removed } });
      showMessage($_('scan.scanComplete'), msg);
    } catch (e) { showMessage($_('scan.scanFailed'), `${e}`); }
    finally { isScanning = false; }
  }

  function openAnimeDetail(anime: Anime) { selectedAnime = anime; currentView = "detail"; }
  function goBack() { currentView = "library"; selectedAnime = null; }
  function openSettings() { currentView = "settings"; }
  async function onSettingsSaved(path: string) { libraryPath = path; currentView = "library"; await loadAnimeList(); }
</script>

<div class="app-shell">
  <TitleBar 
    isDark={isDark} 
    onToggleTheme={toggleTheme}
    currentView={currentView}
    bind:searchQuery
    isScanning={isScanning}
    showBack={currentView !== "library"}
    detailTitle={selectedAnime?.title || ""}
    onBack={goBack}
    onScan={scanLibrary}
    onSettings={openSettings}
    pluginToolbarButtons={currentPluginToolbarButtons}
  />

  <main class="main-content">
    {#if currentView === "library"}
      <AnimeLibrary animeList={filteredAnime} onSelect={openAnimeDetail} />
    {:else if currentView === "detail" && selectedAnime}
      <AnimeDetail anime={selectedAnime} onRefresh={loadAnimeList} />
    {:else if currentView === "settings"}
      <Settings 
        initialPath={libraryPath} 
        onSave={onSettingsSaved} 
        isDark={isDark} 
        onToggleTheme={toggleTheme}
        colorScheme={colorScheme}
        onChangeColorScheme={changeColorScheme}
        radiusScale={radiusScale}
        onChangeRadiusScale={applyRadiusScale}
        bgStyle={bgStyle}
        onChangeBgStyle={applyBgStyle}
        glassMode={glassMode}
        onChangeGlassMode={applyGlassMode}
        shadowStyle={shadowStyle}
        onChangeShadowStyle={applyShadowStyle}
        motionLevel={motionLevel}
        onChangeMotionLevel={applyMotionLevel}
        customBgUrl={customBgUrl}
        bgOpacity={bgOpacity}
        onChangeBgOpacity={applyBgOpacity}
        cardOpacity={cardOpacity}
        onChangeCardOpacity={applyCardOpacity}
        titlebarOpacity={titlebarOpacity}
        onChangeTitlebarOpacity={applyTitlebarOpacity}
      />
    {/if}
  </main>
</div>

{#if showModal}
  <div class="modal-overlay" on:click={closeModal}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3 class="modal-title">{modalTitle}</h3>
        <button class="modal-close" on:click={closeModal}>
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
      <div class="modal-body" bind:this={modalBodyEl}>
        {#if !modalContentHtml}
          <p>{modalMessage}</p>
        {/if}
      </div>
      <div class="modal-footer">
        {#if showConfirm}
          <button class="modal-cancel" on:click={closeModal}>{$_('app.cancel')}</button>
          <button class="modal-confirm" on:click={handleConfirm}>{$_('app.confirm')}</button>
        {:else}
          <button class="modal-confirm" on:click={closeModal}>{$_('app.ok')}</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .app-shell { display: flex; flex-direction: column; height: 100vh; }

  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-8) var(--space-8) var(--space-12);
    scrollbar-gutter: stable;
  }

  .main-content::-webkit-scrollbar {
    width: 6px;
  }

  .main-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .main-content::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
    transition: background 0.2s ease;
  }

  .main-content::-webkit-scrollbar-thumb:hover {
    background: var(--border-strong);
  }
</style>
