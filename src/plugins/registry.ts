import type { PluginManifest, PluginState, PanelOptions, ToolbarButtonOptions, ContextMenuOptions } from './types';

export const plugins = new Map<string, PluginState>();
export const panels = new Map<string, PanelOptions & { pluginId: string }>();
export const toolbarButtons = new Map<string, ToolbarButtonOptions & { pluginId: string }>();
export const contextMenus = new Map<string, ContextMenuOptions & { pluginId: string }>();

let _showModalResolve: ((value: any) => void) | null = null;

export function showModal(options: {
  title: string;
  content: string;
  buttons?: { label: string; type?: string; onClick: () => void | Promise<void> }[];
}): Promise<any> {
  return new Promise((resolve) => {
    _showModalResolve = resolve;
    const w = window as any;
    w.__pluginShowModal?.(options);
  });
}

export function resolveModal(result?: any) {
  _showModalResolve?.(result);
  _showModalResolve = null;
}

export function showToast(options: { message: string; type?: string; duration?: number }) {
  const w = window as any;
  w.__pluginShowToast?.(options);
}

export function getPluginState(id: string): PluginState | undefined {
  return plugins.get(id);
}

export function getAllPlugins(): PluginState[] {
  return Array.from(plugins.values());
}

export function getEnabledPlugins(): PluginState[] {
  return getAllPlugins().filter(p => p.enabled && p.loaded);
}

export function setPluginState(id: string, state: Partial<PluginState>) {
  const existing = plugins.get(id);
  if (existing) {
    plugins.set(id, { ...existing, ...state });
  }
}

export function addPlugin(manifest: PluginManifest) {
  const enabled = localStorage.getItem(`plugin:${manifest.id}:enabled`);
  plugins.set(manifest.id, {
    id: manifest.id,
    manifest,
    enabled: enabled !== 'false',
    loaded: false,
    instance: null,
    error: null,
  });
}

export function removePlugin(id: string) {
  const state = plugins.get(id);
  if (state?.instance?.onUnload) {
    try { state.instance.onUnload(); } catch (e) { console.error(e); }
  }
  plugins.delete(id);
  panels.forEach((v, k) => { if (v.pluginId === id) panels.delete(k); });
  toolbarButtons.forEach((v, k) => { if (v.pluginId === id) toolbarButtons.delete(k); });
  contextMenus.forEach((v, k) => { if (v.pluginId === id) contextMenus.delete(k); });
  localStorage.removeItem(`plugin:${id}:enabled`);
}

export function togglePlugin(id: string) {
  const state = plugins.get(id);
  if (!state) return;
  state.enabled = !state.enabled;
  localStorage.setItem(`plugin:${id}:enabled`, String(state.enabled));
}
