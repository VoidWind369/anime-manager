<script lang="ts">
  import { invoke } from "../utils/tauri-adapter";
  import { _ } from "svelte-i18n";

  export let isDark: boolean;
  export let onToggleTheme: () => void;
  export let currentView: "library" | "detail" | "settings";
  export let searchQuery: string;
  export let isScanning: boolean;
  export let showBack: boolean;
  export let detailTitle: string;
  export let onBack: () => void;
  export let onScan: () => void;
  export let onSettings: () => void;
  export let pluginToolbarButtons: { icon: string; title: string; onClick: () => void }[] = [];

  let showPluginMenu = false;

  function handlePluginClick(btn: { icon: string; title: string; onClick: () => void }) {
    showPluginMenu = false;
    btn.onClick();
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.plugin-menu-wrap')) {
      showPluginMenu = false;
    }
  }

  $: if (showPluginMenu) {
    setTimeout(() => document.addEventListener('click', handleClickOutside, { once: true }), 0);
  }

  async function minimizeWindow() {
    try { await invoke("minimize_window"); } catch (_) {}
  }

  async function maximizeWindow() {
    try { await invoke("toggle_maximize"); } catch (_) {}
  }

  async function closeWindow() {
    try { await invoke("close_window"); } catch (_) {}
  }
</script>

<header class="custom-title-bar" data-tauri-drag-region>
  <div class="title-bar-left">
    {#if showBack}
    <button 
      class="back-btn" 
      on:click={onBack} 
      aria-label={$_('titlebar.back')}
    >
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
    </button>
    {/if}
    <div class="app-icon">
      <svg viewBox="0 0 128 128" width="16" height="16">
        <rect width="128" height="128" rx="28" style="fill: var(--accent-500)"/>
        <path d="M44 40 L84 40 L84 88 L44 88 Z" fill="white" opacity="0.9"/>
        <circle cx="56" cy="52" r="4" style="fill: var(--accent-500)"/>
        <circle cx="72" cy="52" r="4" style="fill: var(--accent-500)"/>
        <path d="M54 68 Q64 78 74 68" style="stroke: var(--accent-500)" stroke-width="3" fill="none" stroke-linecap="round"/>
      </svg>
    </div>
    <span class="app-title">
      {#if currentView === "library"}{$_('app.title')}{:else if currentView === "detail"}{detailTitle}{:else}{$_('titlebar.settings')}{/if}
    </span>
  </div>

  {#if currentView === "library"}
    <div class="title-bar-center">
      <div class="search-box">
        <svg class="search-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
        <input type="text" placeholder={$_('titlebar.search')} bind:value={searchQuery} />
      </div>
    </div>
  {/if}

  <div class="title-bar-right">
    {#if currentView === "library"}
      <button on:click={onScan} disabled={isScanning} class="scan-btn" aria-label={isScanning ? $_('scan.scanning') : $_('scan.scanLibrary')}>
        {#if isScanning}
          <span class="spinner"></span>
        {:else}
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-6.219-8.56"/><path d="M21 3v6h-6"/></svg>
        {/if}
      </button>
    {/if}
    {#if currentView !== "settings"}
      <button class="ghost-icon-btn" on:click={onSettings} aria-label={$_('titlebar.settings')}>
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l-.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.01.6.46 1.1 1 1.2h.09a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
      </button>
    {/if}
    {#if pluginToolbarButtons.length > 0}
      <div class="plugin-menu-wrap">
        <button class="ghost-icon-btn plugin-trigger" on:click|stopPropagation={() => showPluginMenu = !showPluginMenu} title="插件">
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>
        </button>
        {#if showPluginMenu}
          <div class="plugin-dropdown">
            {#each pluginToolbarButtons as btn}
              <button class="plugin-dropdown-item" on:click={() => handlePluginClick(btn)}>
                <span class="plugin-dropdown-icon">{@html btn.icon}</span>
                <span class="plugin-dropdown-label">{btn.title}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
    <button class="ghost-icon-btn theme-toggle" on:click={onToggleTheme} aria-label={isDark ? $_('titlebar.toggleLight') : $_('titlebar.toggleDark')}>
      {#if isDark}
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2"/><path d="M12 21v2"/><path d="m4.22 4.22 1.42 1.42"/><path d="m18.36 18.36 1.42 1.42"/><path d="M1 12h2"/><path d="M21 12h2"/><path d="m6.34 17.66-1.42 1.42"/><path d="m19.78 4.22-1.42 1.42"/></svg>
      {:else}
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
      {/if}
    </button>

    <button class="window-btn minimize" on:click={minimizeWindow} aria-label={$_('titlebar.minimize')}>
      <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12h16"/></svg>
    </button>
    <button class="window-btn maximize" on:click={maximizeWindow} aria-label={$_('titlebar.maximize')}>
      <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg>
    </button>
    <button class="window-btn close" on:click={closeWindow} aria-label={$_('titlebar.close')}>
      <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
    </button>
  </div>
</header>

<style>
  .custom-title-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 48px;
    padding: 0 12px;
    position: relative;
    border-bottom: none;
    user-select: none;
    cursor: default;
  }
  .custom-title-bar::before {
    content: "";
    position: absolute;
    inset: 0;
    background: var(--titlebar-bg);
    opacity: var(--titlebar-opacity, 1);
    z-index: -1;
  }
  .custom-title-bar button {
    background: transparent !important;
    box-shadow: none !important;
    padding: 0 !important;
    border: none !important;
    border-radius: var(--radius-sm) !important;
    color: var(--text-secondary) !important;
    transform: none !important;
  }

  .custom-title-bar button:hover {
    transform: none !important;
    box-shadow: none !important;
    background: rgba(var(--accent-rgb), 0.08) !important;
  }

  .title-bar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(var(--accent-rgb), 0.08);
    color: var(--accent-500);
  }

  .app-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app-icon svg {
    fill: var(--text-primary);
  }

  .app-title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    height: 100%;
  }

  .title-bar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    padding: 0 20px;
    max-width: 400px;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    color: var(--text-tertiary);
  }

  .search-box input {
    width: 100%;
    height: 32px;
    padding: 0 12px 0 36px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    font-family: inherit;
    background: var(--surface);
    color: var(--text-primary);
    outline: none;
    transition: all 0.2s ease;
  }

  .search-box input:focus {
    border-color: var(--accent-400);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.1);
  }

  .search-box input::placeholder {
    color: var(--text-tertiary);
  }

  .title-bar-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .scan-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none !important;
    border-radius: var(--radius-sm) !important;
    padding: 0 !important;
    font-family: inherit;
    cursor: pointer;
    background: transparent !important;
    color: var(--text-secondary) !important;
    box-shadow: none !important;
    transition: all 0.2s ease;
  }

  .scan-btn:hover:not(:disabled) {
    background: rgba(var(--accent-rgb), 0.08) !important;
    color: var(--accent-500) !important;
    transform: none !important;
    box-shadow: none !important;
  }

  .scan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(var(--accent-rgb), 0.3);
    border-top-color: var(--accent-500);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .ghost-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .ghost-icon-btn:hover {
    background: rgba(var(--accent-rgb), 0.08);
    color: var(--accent-500);
  }

  .plugin-menu-wrap {
    position: relative;
  }

  .plugin-trigger :global(svg) {
    width: 15px;
    height: 15px;
  }

  .plugin-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 160px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 100;
    padding: 4px;
    animation: dropdownIn 0.15s ease;
  }

  @keyframes dropdownIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .plugin-dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.15s ease;
    color: var(--text-primary);
    font-size: 0.85rem;
  }

  .plugin-dropdown-item:hover {
    background: rgba(var(--accent-rgb), 0.08);
  }

  .plugin-dropdown-icon :global(svg) {
    width: 15px;
    height: 15px;
    color: var(--text-secondary);
  }

  .plugin-dropdown-item:hover .plugin-dropdown-icon :global(svg) {
    color: var(--accent-500);
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .theme-toggle:hover {
    background: rgba(var(--accent-rgb), 0.08);
    color: var(--accent-500);
  }

  .window-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .window-btn:hover {
    background: rgba(var(--accent-rgb), 0.08);
    color: var(--text-primary);
  }

  .window-btn.close:hover {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-600));
    color: white;
  }

  :global([data-theme="dark"]) .custom-title-bar::before {
    background: var(--titlebar-bg-dark);
  }
  :global([data-theme="dark"]) .custom-title-bar {
    border-bottom-color: var(--border);
  }

  :global([data-theme="dark"]) .app-icon svg {
    fill: var(--text-primary);
  }

  :global([data-theme="dark"]) .app-title {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .back-btn {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .back-btn:hover {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .search-box input {
    background: var(--surface);
    border-color: var(--border);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .search-box input:focus {
    border-color: var(--accent-400);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.15);
  }

  :global([data-theme="dark"]) .search-box input::placeholder {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .scan-btn:hover:not(:disabled) {
    background: rgba(var(--accent-rgb), 0.15) !important;
    color: var(--accent-400) !important;
  }

  :global([data-theme="dark"]) .ghost-icon-btn {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .ghost-icon-btn:hover {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .theme-toggle {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .theme-toggle:hover {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .window-btn {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .window-btn:hover {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .window-btn.close:hover {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-600));
    color: white;
  }

  :global([data-theme="dark"]) .plugin-dropdown {
    background: var(--surface);
    border-color: var(--border);
  }

  :global([data-theme="dark"]) .plugin-dropdown-item {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .plugin-dropdown-item:hover {
    background: rgba(var(--accent-rgb), 0.15);
  }

  :global([data-theme="dark"]) .plugin-dropdown-icon :global(svg) {
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .plugin-dropdown-item:hover .plugin-dropdown-icon :global(svg) {
    color: var(--accent-300);
  }
</style>
