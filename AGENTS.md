# Anime Manager — 本地动漫资源管理器

基于 Tauri + Svelte + Rust 桌面应用，用于管理本地动漫资源库。

## 技术栈

- **前端**: Svelte 4 + TypeScript + Vite 5
- **后端**: Rust + Tauri 2.0
- **数据库**: SQLite (sqlx)
- **UI 风格**: 粉色基底 + 简约风格 + 渐变元素

## 目录结构

```
anime-manager/
├── src/                          # 前端 (Svelte)
│   ├── components/
│   │   ├── TitleBar.svelte       # 自定义标题栏（窗口控制 + 搜索 + 主题切换）
│   │   ├── AnimeLibrary.svelte    # 动漫库主页（卡片网格）
│   │   ├── AnimeDetail.svelte     # 动漫详情页（季/版本选项卡 + 剧集列表）
│   │   └── Settings.svelte       # 设置页
│   ├── types/
│   │   └── anime.ts             # TypeScript 类型定义
│   ├── utils/
│   │   ├── image.ts             # 图片加载与缩略图生成
│   │   └── tauri-adapter.ts     # Tauri API 适配层（兼容浏览器模式）
│   ├── App.svelte               # 根组件
│   ├── main.ts                   # 入口
│   └── styles.css               # 全局样式（设计令牌系统 + 暗色模式）
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── db/
│   │   │   ├── anime.rs         # 动漫/剧集数据库操作
│   │   │   └── mod.rs           # 数据库初始化
│   │   ├── commands.rs          # Tauri 命令接口
│   │   ├── models.rs            # 数据模型
│   │   ├── scanner.rs           # 本地目录扫描器
│   │   ├── organizer.rs         # 目录重命名/整理
│   │   └── lib.rs               # Tauri 应用入口
│   ├── migrations/                # SQL 迁移文件
│   │   ├── 001_init.sql
│   │   ├── 002_add_tracker.sql
│   │   └── 003_add_subtitle.sql
│   └── Cargo.toml
└── package.json
```

## 数据库结构

### anime 表 — 动漫信息

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增 ID |
| title | TEXT | 标题（番剧名） |
| original_title | TEXT | 原名/原始目录名 |
| subtitle | TEXT | 副标题（剧场版/OVA/番外篇等子标题） |
| season | INTEGER | 季数，默认 1 |
| subtitle_group | TEXT | 字幕组 |
| directory_path | TEXT UNIQUE | 目录路径 |
| cover_image | TEXT | 封面图片路径 |
| description | TEXT | 描述 |
| total_episodes | INTEGER | 总集数 |
| watched_episodes | INTEGER | 已看集数 |
| is_movie | INTEGER | 是否剧场版 |
| added_at / updated_at | DATETIME | |

### episodes 表 — 剧集

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | |
| anime_id | INTEGER FK | 关联 anime.id |
| title | TEXT | 剧集标题（文件名） |
| file_path | TEXT UNIQUE | 文件路径 |
| episode_number | INTEGER | 集数（按文件名排序从1开始） |
| duration | REAL | 时长 |
| file_size | INTEGER | 文件大小 |
| watched | INTEGER | 是否已看 |
| watch_progress | REAL | 观看进度 |
| last_watched_at | DATETIME | 最后观看时间 |

### settings 表 — 设置

| 字段 | 类型 | 说明 |
|------|------|------|
| key | TEXT PK | 设置键 |
| value | TEXT | 设置值 |

## 核心功能

### 1. 动漫库扫描 (scanner.rs)

- 扫描 `E:\动漫` 目录（可配置）下的所有动漫目录
- 识别分组父目录（直接子层无视频、孙子目录有视频 → 识别为分组父目录，子目录分别作为独立动漫
- 视频文件扩展名：mp4, mkv, avi, mov, wmv, flv, webm, rmvb, m4v
- 集数编号：按文件名排序从 1 开始递增
- 扫描时自动清理已不存在的目录记录
- 字幕组提取：跳过画质标签（WEB-DL、1080p、HEVC 等），取第一个真正的字幕组

### 2. 目录整理 (organizer.rs)

- 目录命名格式：`[中文标题] 第X季 [字幕组]`
- 多季动漫：父目录（番剧名）→ 子目录（各季）
- 单季动漫和独立剧场版：留在根目录
- 原始目录名备份：每个目录内 `原目录名.txt`
- 已是标准中文名或缓存目录不重命名

### 3. 图片加载 (utils/image.ts)

- 读取本地图片文件，自动检测 MIME 类型（通过文件头字节）
- 大文件分块处理（8192 字节/块），避免调用栈溢出
- Canvas 生成缩略图（卡片 2 倍像素：400×533，JPEG 质量 0.85）
- 无内存 Map 缓存（按需读取，依赖浏览器 `<img>` 解码缓存）

### 4. 自定义标题栏 (TitleBar.svelte)

- 系统标题栏已移除（`decorations: false`，`transparent: true`）
- 标题栏集成：应用图标 + 标题、搜索框、扫描按钮、设置按钮、主题切换、窗口控制按钮
- 窗口控制：最小化、最大化/还原、关闭（通过后端 Tauri 命令实现）
- 拖拽移动窗口（`data-tauri-drag-region`）
- 返回按钮仅在详情页和设置页显示
- 标题栏按钮使用透明背景图标风格，扫描按钮与其他图标按钮风格一致
- 标题栏背景使用不透明渐变，透明度通过 `--titlebar-opacity` 独立可调（0%~100%）
- `::before` 伪元素承载背景和 opacity，避免透明度影响文字

## UI 设计规范

### 设计令牌 (styles.css)

- 粉色色系：--pink-50 到 --pink-700
- 间距系统：--space-1 (4px) 到 --space-12 (48px)
- 圆角：--radius-sm (10px) / md (14px) / lg (20px) / pill (999px)
- 阴影：--shadow-sm / md / lg
- 文本：--text-primary / secondary / tertiary
- 暗色模式：--surface-dark, --text-primary-dark 等变量

### 布局规则

- 元素间距合理，不顶格
- 标题最多显示 2 行，溢出省略号（-webkit-line-clamp: 2）
- 集数和字幕组名完整显示不截断
- 已看剧集用降低透明度表示（不用删除线）
- 不用文字当图片占位符，用渐变色块 + SVG 图标
- 封面图片必须含有关键人物，不含纯 logo/文字标题图

### 交互规则

- 全局禁用文本选取（`user-select: none`），仅输入框和 `.copyable` 元素可选取
- 全局禁用右键菜单（`contextmenu` 事件阻止）
- 全局禁用 `-webkit-touch-callout`（移动端长按菜单）
- `.copyable` 类：hover 时文字变为粉色（`--accent-500`），active 时变为深粉色（`--accent-600`），不用下划线
- 动漫标题（卡片标题、详情页标题）和剧集文件名添加 `.copyable` 类，点击复制到剪贴板

### 主页 (AnimeLibrary.svelte)

- 按番剧名分组，一部动漫只显示一张卡片
- 卡片左上角「X 季」角标（多季时）
- 同季多版本显示「X 个版本」
- 网格布局：auto-fill + minmax(200px, 1fr)
- 卡片标题点击可复制

### 详情页 (AnimeDetail.svelte)

- 顶部：封面 + 标题 + 季标签 + 集数统计
- 字幕组标签（.tag-sub）：毛玻璃效果（半透明背景 + backdrop-filter + 边框），暗色模式下使用 `var(--accent-200)` 颜色
- 季选项卡：扁平布局（无卡片框），胶囊式，粉色渐变选中态
- 版本选项卡：扁平布局（无卡片框），横向切换字幕组/版本
- 剧集列表：当前选中版本的剧集，每行编号 + 标题 + 播放/已看按钮
- 标题和剧集文件名点击可复制
- 打开目录按钮通过后端 `open_directory` 命令实现（Windows 使用 `cmd /C start`）

### 设置页 (Settings.svelte)

- 选项卡式布局：通用 / 外观 / 关于
- 选项卡样式：扁平集成式，底部边框指示器（`border-bottom: 2px solid var(--accent-500)`）
- **通用**：库目录、界面语言
- **外观**：主题、配色方案、圆角、背景、透明度/毛玻璃、阴影、动画
- **关于**：支持格式、命名规则、插件管理

## Tauri 命令列表

| 命令 | 说明 |
|------|------|
| get_anime_list | 获取所有动漫列表 |
| get_anime_by_title | 按标题获取同部动漫所有版本 |
| get_anime_episodes | 获取动漫的剧集列表 |
| scan_library | 扫描动漫库 |
| play_episode | 播放剧集 |
| toggle_episode_watched | 切换剧集已看状态 |
| get_library_path / set_library_path | 获取/设置库路径 |
| get_original_name | 获取原始目录名 |
| delete_anime | 删除动漫 |
| rename_anime_directory | 重命名动漫目录 |
| organize_multi_season | 整理多季动漫 |
| open_directory | 打开目录（系统文件管理器） |
| minimize_window | 最小化窗口 |
| toggle_maximize | 切换最大化/还原 |
| close_window | 关闭窗口 |

## 开发命令

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 仅前端开发
npm run dev
```

## 重要约束

- 动漫库路径默认：`E:\动漫`
- 数据库文件：应用数据目录下的 `anime-manager.db`
- 集数编号：按文件名排序，从 1 开始
- 封面图片：仅从本地目录扫描（cover.jpg/png/webp/gif、poster.jpg/png、folder.jpg/png）
- 封面必须含有人物，不含 logo
- 不使用文字删除线，已看用降低透明度
- 前端不使用过多缓存机制（按需读取）
- 支持暗色模式，所有 UI 元素需适配
- 所有浏览器默认弹窗(alert)已替换为自定义模态框
- Tauri API 适配层支持浏览器模式（tauri-adapter.ts）
- 系统标题栏已移除，使用自定义标题栏（TitleBar.svelte）
- 全局禁用文本选取和右键菜单，仅 `.copyable` 元素可选取和复制
- 窗口控制（最小化/最大化/关闭）通过后端 Tauri 命令实现
- 仅保留渐变按钮风格（gradient），不再支持扁平/描边等其他风格
- **禁止擅自删除或削减动画效果**（hover 变色过渡、展开/收起动画、translateY 等），修改动画必须经用户确认
- 每次完成开发后检测应用启动状态，未启动则主动启动


