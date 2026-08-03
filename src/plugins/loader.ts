import type { PluginManifest, PluginInstance, PluginContext } from './types';
import { createPluginContext } from './api';
import { addPlugin, plugins, setPluginState } from './registry';

const PLUGIN_BASE_PATH = 'plugins';

function getPluginsPath(): string {
  const w = window as any;
  return w.__pluginsDir ?? `${PLUGIN_BASE_PATH}`;
}

function normalizeManifest(raw: any): PluginManifest {
  return {
    id: raw.id ?? raw.name ?? 'unknown',
    name: raw.name ?? raw.id ?? 'Unknown Plugin',
    version: raw.version ?? '0.0.0',
    description: raw.description ?? '',
    author: raw.author ?? 'Unknown',
    main: raw.main ?? 'index.js',
    settings: raw.settings ?? [],
    hooks: raw.hooks ?? [],
  };
}

export async function loadAllPlugins(): Promise<void> {
  const basePath = getPluginsPath();
  let pluginDirs: string[] = [];

  try {
    const { readDir } = await import('@tauri-apps/plugin-fs');
    const entries = await readDir(basePath);
    pluginDirs = entries
      .filter(e => e.isDirectory)
      .map(e => e.name ?? '')
      .filter(n => n.length > 0);
  } catch (e) {
    console.warn('[PluginLoader] Failed to read plugins directory:', e);
    return;
  }

  for (const dir of pluginDirs) {
    try {
      await loadPlugin(basePath, dir);
    } catch (e) {
      console.error(`[PluginLoader] Failed to load plugin "${dir}":`, e);
    }
  }
}

async function loadPlugin(basePath: string, dir: string): Promise<void> {
  const manifestPath = `${basePath}\\${dir}\\manifest.json`;

  let manifestRaw: any;
  try {
    const { readFile } = await import('@tauri-apps/plugin-fs');
    const bytes = await readFile(manifestPath);
    const text = new TextDecoder().decode(bytes);
    manifestRaw = JSON.parse(text);
  } catch (e) {
    console.warn(`[PluginLoader] No manifest.json in "${dir}", skipping.`);
    return;
  }

  const manifest = normalizeManifest(manifestRaw);
  addPlugin(manifest);

  const state = plugins.get(manifest.id);
  if (state && !state.enabled) {
    console.log(`[PluginLoader] Plugin "${manifest.id}" is disabled, skipping load.`);
    return;
  }

  await instantiatePlugin(basePath, dir, manifest);
}

async function instantiatePlugin(basePath: string, dir: string, manifest: PluginManifest): Promise<void> {
  const entryPath = `${basePath}\\${dir}\\${manifest.main}`;
  const pluginCtx: PluginContext = createPluginContext(manifest.id);

  (window as any).__pluginBasePath = (id: string) => `${basePath}\\${id}`;

  try {
    const moduleUrl = `file:///${entryPath.replace(/\\/g, '/')}?t=${Date.now()}`;
    const mod = await import(/* @vite-ignore */ moduleUrl);

    const instance: PluginInstance = {
      onLoad: mod.onLoad,
      onUnload: mod.onUnload,
      onEnable: mod.onEnable,
      onDisable: mod.onDisable,
      onAnimeSelect: mod.onAnimeSelect,
      onBeforeScan: mod.onBeforeScan,
      onAfterScan: mod.onAfterScan,
      onSettingsChange: mod.onSettingsChange,
    };

    setPluginState(manifest.id, { instance, loaded: true, error: null });

    if (instance.onLoad) {
      await instance.onLoad(pluginCtx);
    }

    console.log(`[PluginLoader] Plugin "${manifest.id}" loaded successfully.`);
  } catch (e: any) {
    setPluginState(manifest.id, { error: e.message ?? String(e), loaded: false });
    console.error(`[PluginLoader] Plugin "${manifest.id}" failed to load:`, e);
  }
}

export async function enablePlugin(id: string): Promise<void> {
  const state = plugins.get(id);
  if (!state || state.enabled) return;

  state.enabled = true;
  localStorage.setItem(`plugin:${id}:enabled`, 'true');

  if (!state.loaded) {
    const basePath = getPluginsPath();
    const dir = id;
    await instantiatePlugin(basePath, dir, state.manifest);
  } else if (state.instance?.onEnable) {
    const ctx = createPluginContext(id);
    await state.instance.onEnable();
  }
}

export async function disablePlugin(id: string): Promise<void> {
  const state = plugins.get(id);
  if (!state || !state.enabled) return;

  if (state.instance?.onDisable) {
    try { await state.instance.onDisable(); } catch (e) { console.error(e); }
  }

  state.enabled = false;
  localStorage.setItem(`plugin:${id}:enabled`, 'false');
}

export async function uninstallPlugin(id: string): Promise<void> {
  const state = plugins.get(id);
  if (!state) return;

  if (state.instance?.onUnload) {
    try { await state.instance.onUnload(); } catch (e) { console.error(e); }
  }

  const basePath = getPluginsPath();
  try {
    const { remove } = await import('@tauri-apps/plugin-fs');
    await remove(`${basePath}\\${id}`, { recursive: true });
  } catch (e) {
    console.error(`[PluginLoader] Failed to delete plugin directory "${id}":`, e);
  }

  const { removePlugin: removePluginFromRegistry } = await import('./registry');
  removePluginFromRegistry(id);
}

export function fireAnimeSelect(anime: any) {
  const { getEnabledPlugins } = require('./registry');
  for (const state of getEnabledPlugins()) {
    if (state.instance?.onAnimeSelect) {
      try { state.instance.onAnimeSelect(anime); } catch (e) { console.error(e); }
    }
  }
}

export function fireBeforeScan(path: string) {
  const { getEnabledPlugins } = require('./registry');
  for (const state of getEnabledPlugins()) {
    if (state.instance?.onBeforeScan) {
      try { state.instance.onBeforeScan(path); } catch (e) { console.error(e); }
    }
  }
}

export function fireAfterScan(result: any) {
  const { getEnabledPlugins } = require('./registry');
  for (const state of getEnabledPlugins()) {
    if (state.instance?.onAfterScan) {
      try { state.instance.onAfterScan(result); } catch (e) { console.error(e); }
    }
  }
}
