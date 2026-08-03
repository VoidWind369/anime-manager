import type { Anime, Episode, ScanResult } from '../types/anime';

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  main: string;
  settings?: PluginSettingDef[];
  hooks?: string[];
}

export interface PluginSettingDef {
  key: string;
  label: string;
  type: 'text' | 'password' | 'number' | 'boolean' | 'select';
  default?: any;
  options?: { label: string; value: string }[];
  placeholder?: string;
}

export interface PluginState {
  id: string;
  manifest: PluginManifest;
  enabled: boolean;
  loaded: boolean;
  instance: PluginInstance | null;
  error: string | null;
}

export interface PluginInstance {
  onLoad?: (ctx: PluginContext) => void | Promise<void>;
  onUnload?: () => void | Promise<void>;
  onEnable?: () => void | Promise<void>;
  onDisable?: () => void | Promise<void>;
  onAnimeSelect?: (anime: Anime) => void | Promise<void>;
  onBeforeScan?: (path: string) => void | Promise<void>;
  onAfterScan?: (result: ScanResult) => void | Promise<void>;
  onSettingsChange?: (settings: Record<string, any>) => void | Promise<void>;
}

export interface PluginContext {
  app: PluginAppAPI;
  ui: PluginUIAPI;
  backend: PluginBackendAPI;
  storage: PluginStorageAPI;
  log: PluginLogAPI;
}

export interface PluginAppAPI {
  getAnimeList(): Anime[];
  getSettings(): Record<string, any>;
  navigate(view: string, data?: any): void;
  getPluginPath(): string;
}

export interface PluginUIAPI {
  addPanel(id: string, options: PanelOptions): void;
  removePanel(id: string): void;
  addToolbarButton(id: string, options: ToolbarButtonOptions): void;
  removeToolbarButton(id: string): void;
  addContextMenu(id: string, options: ContextMenuOptions): void;
  removeContextMenu(id: string): void;
  showModal(options: ModalOptions): Promise<any>;
  showToast(options: ToastOptions): void;
}

export interface PanelOptions {
  title: string;
  icon?: string;
  position?: 'sidebar' | 'main' | 'tab';
  order?: number;
  render: (container: HTMLElement) => void;
  destroy?: () => void;
}

export interface ToolbarButtonOptions {
  icon: string;
  title: string;
  position?: 'left' | 'right';
  order?: number;
  onClick: () => void;
}

export interface ContextMenuOptions {
  label: string;
  icon?: string;
  position?: 'top' | 'bottom';
  order?: number;
  onClick: (data?: any) => void;
}

export interface ModalOptions {
  title: string;
  content: string | HTMLElement;
  buttons?: ModalButton[];
  width?: number;
  height?: number;
}

export interface ModalButton {
  label: string;
  type?: 'primary' | 'secondary' | 'danger';
  onClick: () => void | Promise<void>;
}

export interface ToastOptions {
  message: string;
  type?: 'info' | 'success' | 'warning' | 'error';
  duration?: number;
}

export interface PluginBackendAPI {
  invoke(command: string, args?: Record<string, any>): Promise<any>;
  on(event: string, handler: (...args: any[]) => void): void;
  off(event: string, handler: (...args: any[]) => void): void;
}

export interface PluginStorageAPI {
  get(key: string): any;
  set(key: string, value: any): void;
  remove(key: string): void;
  clear(): void;
}

export interface PluginLogAPI {
  info(msg: string): void;
  warn(msg: string): void;
  error(msg: string): void;
}
