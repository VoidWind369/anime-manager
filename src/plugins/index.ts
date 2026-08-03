export type {
  PluginManifest,
  PluginState,
  PluginInstance,
  PluginContext,
  PluginSettingDef,
  PanelOptions,
  ToolbarButtonOptions,
  ContextMenuOptions,
  ModalOptions,
  ToastOptions,
} from './types';

export {
  plugins,
  panels,
  toolbarButtons,
  contextMenus,
  getAllPlugins,
  getEnabledPlugins,
  getPluginState,
  addPlugin,
  removePlugin,
  togglePlugin,
  showModal,
  resolveModal,
  showToast,
} from './registry';

export {
  loadAllPlugins,
  enablePlugin,
  disablePlugin,
  uninstallPlugin,
  fireAnimeSelect,
  fireBeforeScan,
  fireAfterScan,
} from './loader';

export { createPluginContext } from './api';
