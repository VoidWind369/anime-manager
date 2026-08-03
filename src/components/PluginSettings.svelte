<script lang="ts">
  import { _ } from "svelte-i18n";
  import { getAllPlugins, togglePlugin, uninstallPlugin, type PluginState } from "../plugins/index";

  export let onBack: () => void;

  let pluginList: PluginState[] = [];
  let showConfirmModal = false;
  let confirmTarget: PluginState | null = null;
  let showModal = false;
  let modalTitle = "";
  let modalMessage = "";

  function refreshList() {
    pluginList = getAllPlugins();
  }

  function showMessage(title: string, message: string) {
    modalTitle = title;
    modalMessage = message;
    showModal = true;
  }

  refreshList();

  function handleToggle(p: PluginState) {
    togglePlugin(p.id);
    refreshList();
  }

  async function confirmUninstall(p: PluginState) {
    confirmTarget = p;
    showConfirmModal = true;
  }

  async function doUninstall() {
    if (!confirmTarget) return;
    try {
      await uninstallPlugin(confirmTarget.id);
      showMessage($_('plugin.uninstallSuccess'), `${confirmTarget.manifest.name} ${$_('plugin.uninstallSuccess')}`);
    } catch (e) {
      showMessage($_('plugin.uninstallFailed'), `${e}`);
    }
    confirmTarget = null;
    showConfirmModal = false;
    refreshList();
  }
</script>

<div class="plugin-settings">
  <div class="plugin-header">
    <button class="back-btn" on:click={onBack}>
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
    </button>
    <h2>{$_('plugin.title')}</h2>
  </div>

  <p class="plugin-hint">{$_('plugin.hint')}</p>

  {#if pluginList.length === 0}
    <div class="plugin-empty">
      <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
      <p>{$_('plugin.noPlugins')}</p>
      <span>{$_('plugin.noPluginsHint')}</span>
    </div>
  {:else}
    <div class="plugin-list">
      {#each pluginList as p (p.id)}
        <div class="plugin-card" class:disabled={!p.enabled}>
          <div class="plugin-info">
            <div class="plugin-name">{p.manifest.name}</div>
            <div class="plugin-meta">
              <span class="plugin-version">v{p.manifest.version}</span>
              <span class="plugin-author">{p.manifest.author}</span>
            </div>
            <div class="plugin-desc">{p.manifest.description}</div>
            {#if p.error}
              <div class="plugin-error">{p.error}</div>
            {/if}
          </div>
          <div class="plugin-actions">
            <button
              class="toggle-btn"
              class:active={p.enabled}
              on:click={() => handleToggle(p)}
            >
              {p.enabled ? $_('plugin.disable') : $_('plugin.enable')}
            </button>
            <button class="uninstall-btn" on:click={() => confirmUninstall(p)}>
              {$_('plugin.uninstall')}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showModal}
  <div class="modal-overlay" on:click={() => showModal = false}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3 class="modal-title">{modalTitle}</h3>
      </div>
      <div class="modal-body">
        <p>{modalMessage}</p>
      </div>
      <div class="modal-footer">
        <button class="modal-confirm" on:click={() => showModal = false}>{$_('app.ok')}</button>
      </div>
    </div>
  </div>
{/if}

{#if showConfirmModal}
  <div class="modal-overlay" on:click={() => showConfirmModal = false}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3 class="modal-title">{$_('plugin.confirmUninstall')}</h3>
      </div>
      <div class="modal-body">
        <p>{$_('plugin.confirmUninstallMsg', { values: { name: confirmTarget?.manifest.name ?? '' } })}</p>
      </div>
      <div class="modal-footer">
        <button class="modal-cancel" on:click={() => showConfirmModal = false}>{$_('app.cancel')}</button>
        <button class="modal-confirm danger" on:click={doUninstall}>{$_('app.confirm')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .plugin-settings {
    max-width: 640px;
    margin: 0 auto;
  }

  .plugin-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .plugin-header h2 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }

  .back-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
  }

  .back-btn:hover { color: var(--accent-500); }

  .plugin-hint {
    color: var(--text-tertiary);
    font-size: 0.85rem;
    margin-bottom: var(--space-6);
  }

  .plugin-empty {
    text-align: center;
    padding: var(--space-12) 0;
    color: var(--text-tertiary);
  }

  .plugin-empty svg { margin-bottom: var(--space-3); }
  .plugin-empty p { font-size: 1rem; margin: 0 0 var(--space-2); }
  .plugin-empty span { font-size: 0.82rem; }

  .plugin-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .plugin-card {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-4) var(--space-5);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-4);
    transition: all 0.2s ease;
  }

  .plugin-card.disabled { opacity: 0.5; }

  .plugin-info { flex: 1; min-width: 0; }

  .plugin-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 0.95rem;
  }

  .plugin-meta {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-1);
  }

  .plugin-version, .plugin-author {
    font-size: 0.78rem;
    color: var(--text-tertiary);
  }

  .plugin-desc {
    font-size: 0.82rem;
    color: var(--text-secondary);
    margin-top: var(--space-2);
    line-height: 1.4;
  }

  .plugin-error {
    font-size: 0.78rem;
    color: #ef4444;
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius-sm);
  }

  .plugin-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .toggle-btn, .uninstall-btn {
    padding: 6px 14px;
    border-radius: var(--radius-pill);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }

  .toggle-btn.active {
    background: var(--accent-500);
    color: white;
    border-color: var(--accent-500);
  }

  .toggle-btn:hover { background: var(--accent-50); }
  .toggle-btn.active:hover { background: var(--accent-600); }

  .uninstall-btn:hover {
    color: #ef4444;
    border-color: #ef4444;
    background: rgba(239, 68, 68, 0.05);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal-content {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    width: 90%;
    max-width: 400px;
    box-shadow: var(--shadow-lg);
  }

  .modal-header { margin-bottom: var(--space-4); }

  .modal-title {
    margin: 0;
    font-size: 1.05rem;
    color: var(--text-primary);
  }

  .modal-body p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }

  .modal-cancel, .modal-confirm {
    padding: 8px 18px;
    border-radius: var(--radius-pill);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-secondary);
  }

  .modal-confirm {
    background: var(--accent-500);
    color: white;
    border-color: var(--accent-500);
  }

  .modal-confirm.danger {
    background: #ef4444;
    border-color: #ef4444;
  }
</style>
