import type {
  PluginContext,
  PluginAppAPI,
  PluginUIAPI,
  PluginBackendAPI,
  PluginStorageAPI,
  PluginLogAPI,
  PanelOptions,
  ToolbarButtonOptions,
  ContextMenuOptions,
  ModalOptions,
  ToastOptions,
} from './types';
import { panels, toolbarButtons, contextMenus, showModal as appShowModal, showToast as appShowToast } from './registry';

class PluginApp implements PluginAppAPI {
  private pluginId: string;

  constructor(pluginId: string) {
    this.pluginId = pluginId;
  }

  getAnimeList() {
    const w = window as any;
    return w.__pluginAnimeList?.() ?? [];
  }

  getSettings() {
    const w = window as any;
    return w.__pluginSettings?.() ?? {};
  }

  navigate(view: string, data?: any) {
    const w = window as any;
    w.__pluginNavigate?.(view, data);
  }

  getPluginPath() {
    const w = window as any;
    return w.__pluginBasePath?.(this.pluginId) ?? '';
  }
}

class PluginUI implements PluginUIAPI {
  private pluginId: string;

  constructor(pluginId: string) {
    this.pluginId = pluginId;
  }

  addPanel(id: string, options: PanelOptions) {
    panels.set(`${this.pluginId}:${id}`, { ...options, pluginId: this.pluginId });
    const w = window as any;
    w.__pluginPanelsChanged?.();
  }

  removePanel(id: string) {
    panels.delete(`${this.pluginId}:${id}`);
    const w = window as any;
    w.__pluginPanelsChanged?.();
  }

  addToolbarButton(id: string, options: ToolbarButtonOptions) {
    toolbarButtons.set(`${this.pluginId}:${id}`, { ...options, pluginId: this.pluginId });
    const w = window as any;
    w.__pluginToolbarChanged?.();
  }

  removeToolbarButton(id: string) {
    toolbarButtons.delete(`${this.pluginId}:${id}`);
    const w = window as any;
    w.__pluginToolbarChanged?.();
  }

  addContextMenu(id: string, options: ContextMenuOptions) {
    contextMenus.set(`${this.pluginId}:${id}`, { ...options, pluginId: this.pluginId });
  }

  removeContextMenu(id: string) {
    contextMenus.delete(`${this.pluginId}:${id}`);
  }

  async showModal(options: ModalOptions) {
    return appShowModal(options);
  }

  showToast(options: ToastOptions) {
    appShowToast(options);
  }
}

class PluginBackend implements PluginBackendAPI {
  async invoke(command: string, args?: Record<string, any>) {
    const { invoke } = await import('../utils/tauri-adapter');
    return invoke(command, args);
  }

  on(event: string, handler: (...args: any[]) => void) {
    const w = window as any;
    w.__pluginEventsOn?.(event, handler);
  }

  off(event: string, handler: (...args: any[]) => void) {
    const w = window as any;
    w.__pluginEventsOff?.(event, handler);
  }
}

class PluginStorage implements PluginStorageAPI {
  private namespace: string;

  constructor(pluginId: string) {
    this.namespace = `plugin:${pluginId}:`;
  }

  get(key: string): any {
    const raw = localStorage.getItem(this.namespace + key);
    if (raw === null) return null;
    try { return JSON.parse(raw); } catch { return raw; }
  }

  set(key: string, value: any) {
    localStorage.setItem(this.namespace + key, JSON.stringify(value));
  }

  remove(key: string) {
    localStorage.removeItem(this.namespace + key);
  }

  clear() {
    const keys = Object.keys(localStorage).filter(k => k.startsWith(this.namespace));
    keys.forEach(k => localStorage.removeItem(k));
  }
}

class PluginLog implements PluginLogAPI {
  private prefix: string;

  constructor(pluginId: string) {
    this.prefix = `[Plugin:${pluginId}]`;
  }

  info(msg: string) {
    console.log(`${this.prefix} ${msg}`);
  }

  warn(msg: string) {
    console.warn(`${this.prefix} ${msg}`);
  }

  error(msg: string) {
    console.error(`${this.prefix} ${msg}`);
  }
}

export function createPluginContext(pluginId: string): PluginContext {
  return {
    app: new PluginApp(pluginId),
    ui: new PluginUI(pluginId),
    backend: new PluginBackend(),
    storage: new PluginStorage(pluginId),
    log: new PluginLog(pluginId),
  };
}
