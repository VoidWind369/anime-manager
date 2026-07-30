# 修改记录

## 2026-07-30

### 1. 修复类型导入（tauri-adapter.ts）

移除不存在的类型导入 `TrackedAnime`、`UpdateHistory`、`CheckResult`，仅保留 `Anime`、`Episode`、`ScanResult`。

### 2. 修复封面显示条件（AnimeDetail.svelte:194）

封面条件从 `{#if coverDataUrl && anime.cover_image}` 改为 `{#if coverDataUrl}`，避免 `cover_image` 为空但 `coverDataUrl` 已加载时封面不显示。

### 3. 修复 open_directory 编译兼容（commands.rs:75）

添加 `_app: AppHandle` 参数，非 Windows 分支使用 `_app.shell().open()` 替代不存在的 `tauri::App::global()`。

### 4. 修复扫描分组检测逻辑（scanner.rs:307-312）

分组检测中 `break` 改为 `continue`，避免跳过非动漫子目录后的有效子目录。

### 5. 添加 .copyable hover 样式（AnimeLibrary.svelte + AnimeDetail.svelte）

为卡片标题、详情页标题、剧集文件名添加 scoped hover 变色样式，确保 Svelte 4 双 class scoping 下能正确覆盖。

### 6. 字幕组标签暗色模式适配（AnimeDetail.svelte）

`.tag-sub` 暗色模式颜色从 `var(--accent-400)` 改为 `var(--accent-200)`，确保 12 套配色方案下亮暗切换有明显区别。

### 7. 毛玻璃效果（AnimeDetail.svelte）

- `.tag-sub`（字幕组标签）：添加半透明背景 + `backdrop-filter: blur(8px)` + 边框
- `.ep-count`（集数统计）：同样添加毛玻璃效果

### 8. 移除季/版本选项卡卡片框（AnimeDetail.svelte）

移除 `.season-tabs` 和 `.version-tabs` 上的 `background`、`border`、`border-radius`、`padding`、`overflow-x: auto`，同时修复 hover 时按钮顶部被裁切的 bug（`overflow-x: auto` 隐式裁切垂直溢出）。

### 9. 移除按钮风格选项（全局清理）

删除设置页中的「按钮风格」选项，仅保留渐变风格。清理涉及：

- `Settings.svelte`：删除 `buttonStyleOptions`、`buttonStyle` / `onChangeButtonStyle` props、UI 区块、预览图标 CSS
- `App.svelte`：删除 `buttonStyle` 状态变量、`loadSetting("btn-style")`、`applyButtonStyle()` 函数、传递给 Settings 的 props
- `styles.css`：删除整个 `[data-btn-style]` 选择器块（flat + outline 共 57 行）
- `index.html`：删除 `data-btn-style="gradient"` 属性

### 10. 标题栏透明度独立控制（全局）

- 修改 `styles.css` `:root` 默认 `--titlebar-bg` 为半透明渐变，所有玻璃模式下标题栏都透明
- 移除 `translucent` 和 `glass` 模式对 `--titlebar-bg` 的覆盖（仅保留 blur 差异）
- 新增 `--titlebar-opacity` CSS 变量，`TitleBar.svelte` 应用 `opacity: var(--titlebar-opacity)`
- `Settings.svelte` 新增「标题栏透明度」滑块（0% ~ 100%）
- `App.svelte` 新增 `titlebarOpacity` 状态、`applyTitlebarOpacity()` 函数、localStorage 持久化
- 「实色」选项描述从「不透明」改为「无模糊」

### 11. 修复标题栏透明度影响文字（TitleBar.svelte + styles.css）

- 将 `opacity` 从 `.custom-title-bar` 元素移至 `::before` 伪元素，仅控制背景透明度
- 更新 `styles.css` 中毛玻璃模式的标题栏样式，`background` 改为作用于 `::before`
- 更新暗色模式标题栏样式，`background` 同样改为作用于 `::before`

### 12. 修复标题栏透明度100%仍透明的bug（styles.css）

- `:root` 默认 `--titlebar-bg` 从半透明 rgba 渐变改为不透明渐变 `var(--accent-50)` / `var(--accent-100)`
- `--titlebar-opacity` 默认值从 `0.8` 改为 `1`
- `--titlebar-border` 从 `rgba(var(--accent-rgb), 0.15)` 改为 `var(--accent-200)`
- 暗色模式标题栏背景同样改为不透明渐变

之前 `--titlebar-bg` 使用 `rgba(..., 0.08/0.04)` 低 alpha 值，opacity 滑块叠加后仍无法达到完全不透明。修复后 opacity 滑块能正确控制透明度（0%=全透明，100%=全不透明）。
