# Anime Manager

基于 Tauri + Svelte + Rust 的桌面应用，用于管理本地动漫资源库。

## 功能特性

- **动漫库扫描**：自动扫描本地目录，识别动漫、剧场版、多季作品
- **分组管理**：按番剧名分组，显示季数和版本数量
- **封面管理**：自动从本地目录加载封面图片
- **观看进度**：记录已看集数和观看进度
- **目录整理**：一键重命名目录为标准格式
- **多配色方案**：12 套预设主题，支持亮/暗色模式切换
- **自定义标题栏**：集成搜索、扫描、设置、主题切换和窗口控制
- **毛玻璃效果**：支持半透明和玻璃态 UI 风格

## 技术栈

- **前端**: Svelte 4 + TypeScript + Vite 5
- **后端**: Rust + Tauri 2.0
- **数据库**: SQLite (sqlx)
- **UI 风格**: 粉色基底 + 简约风格 + 渐变元素

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建 Release

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/anime-manager.exe`

## 数据库位置

运行后数据库文件自动生成在：

```
C:\Users\<用户名>\AppData\Roaming\com.animemanager.app\anime-manager.db
```

## 目录结构

```
anime-manager/
├── src/                          # 前端 (Svelte)
│   ├── components/
│   │   ├── TitleBar.svelte       # 自定义标题栏
│   │   ├── AnimeLibrary.svelte   # 动漫库主页
│   │   ├── AnimeDetail.svelte    # 动漫详情页
│   │   └── Settings.svelte       # 设置页
│   ├── types/
│   │   └── anime.ts             # TypeScript 类型定义
│   ├── utils/
│   │   ├── image.ts             # 图片加载与缩略图生成
│   │   └── tauri-adapter.ts     # Tauri API 适配层
│   ├── App.svelte               # 根组件
│   ├── main.ts                   # 入口
│   └── styles.css               # 全局样式
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── db/                  # 数据库操作
│   │   ├── commands.rs          # Tauri 命令接口
│   │   ├── scanner.rs           # 本地目录扫描器
│   │   └── organizer.rs         # 目录整理
│   └── Cargo.toml
├── AGENTS.md                     # 项目开发规范
├── NOTES.md                      # 修改记录
└── package.json
```

## 配置

动漫库默认路径：`E:\动漫`（可在设置中修改）

## 许可证

MIT
