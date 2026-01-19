# SmartPaste - 智能剪贴板与桌面助手

> 清华大学软件学院 2025 年软件工程课程项目

SmartPaste 是一款基于 **Tauri** + **Vue 3** + **Rust** 构建的现代化桌面应用。它不仅是一个功能强大的剪贴板管理工具，还集成了可爱的 Live2D 桌面宠物、AI 智能助手以及 OCR、二维码识别等实用工具，旨在提升用户的日常办公效率与交互体验。

## ✨ 主要功能

*   **📋 智能剪贴板管理**
    *   **历史记录**：自动记录复制的文本、图片和文件路径。
    *   **本地存储**：使用 SQLite 数据库本地持久化存储，保障隐私安全。
    *   **搜索与分类**：支持对剪贴板内容进行检索和管理。
    *   **跨设备同步**：(开发中) 支持多设备间的剪贴板同步。

*   **🐾 桌面宠物 (Desktop Pet)**
    *   **Live2D 集成**：内置可爱的 Live2D 模型（基于 PixiJS）。
    *   **交互互动**：支持点击、拖拽等交互反馈，陪伴你的工作时光。

*   **🤖 AI 智能助手**
    *   集成 AI Agent，提供智能问答与辅助功能。
    *   支持上下文理解与多轮对话。

*   **🛠️ 实用工具箱**
    *   **OCR 文字识别**：快速提取图片中的文字信息。
    *   **QR Code 识别**：识别剪贴板或图片中的二维码内容。

## 🛠️ 技术栈

本项目采用前后端分离架构，充分利用了 Web 技术的灵活性和 Rust 的高性能：

*   **前端 (Frontend)**
    *   **Framework**: [Vue 3](https://vuejs.org/) (Composition API, `<script setup>`)
    *   **Build Tool**: [Vite](https://vitejs.dev/)
    *   **Language**: TypeScript / JavaScript
    *   **State Management**: [Pinia](https://pinia.vuejs.org/)
    *   **Router**: Vue Router
    *   **Graphics**: PixiJS & Live2D (pixi-live2d-display)

*   **后端 (Backend)**
    *   **Core**: [Tauri v2](https://tauri.app/)
    *   **Language**: [Rust](https://www.rust-lang.org/)
    *   **Database**: SQLite (via `rusqlite`)
    *   **Libraries**: `arboard` (剪贴板), `image`, `chrono`, `serde`

## 📂 项目结构

```
SmartPaste/
├── public/              # 静态资源 (Live2D 模型等)
├── src/                 # 前端 Vue 源码
│   ├── assets/          # 图片与样式资源
│   ├── components/      # Vue 组件 (DesktopPet, ClipboardApp, Settings 等)
│   ├── composables/     # 组合式函数 (逻辑复用)
│   ├── router/          # 路由配置
│   ├── services/        # API 服务与网络请求
│   ├── stores/          # Pinia 状态管理
│   ├── utils/           # 前端工具函数 (Live2DManager 等)
│   ├── App.vue          # 根组件
│   └── main.ts          # 入口文件
├── src-tauri/           # Rust 后端源码
│   ├── capabilities/    # Tauri 权限配置
│   ├── icons/           # 应用图标
│   ├── resources/       # 后端资源文件
│   ├── src/             
│   │   ├── db/          # 数据库操作逻辑
│   │   ├── ocr.rs       # OCR 模块
│   │   ├── qrcode.rs    # 二维码模块
│   │   ├── clipboard.rs # 剪贴板监听逻辑
│   │   └── main.rs      # Rust 入口
│   ├── Cargo.toml       # Rust 依赖配置
│   └── tauri.conf.json  # Tauri 配置文件
├── index.html           # Web 入口
├── package.json         # Node 依赖配置
└── vite.config.ts       # Vite 配置
```

## 🚀 快速开始

### 环境依赖

在开始之前，请确保你的开发环境已安装以下工具：

1.  **Node.js**: (建议 v18+) [下载](https://nodejs.org/)
2.  **Rust**: (建议使用 rustup 安装) [安装指南](https://www.rust-lang.org/tools/install)
3.  **VS Code 插件推荐**:
    *   Vue - Official
    *   rust-analyzer
    *   Tauri

### 安装依赖

```bash
# 进入项目根目录
cd SmartPaste

# 安装前端依赖
npm install
```

### 开发环境运行

启动 Tauri 开发环境（同时启动前端和后端）：

```bash
npm run tauri dev
```
*首次运行时，Rust 编译可能需要几分钟时间，请耐心等待。*

### 构建生产版本

打包生成可执行文件（Windows .exe / macOS .app）：

```bash
npm run tauri build
```
构建产物将位于 `src-tauri/target/release/bundle/` 目录下。

## ⚠️ 注意事项

*   **Live2D 资源**: 由于版权原因，部分 Live2D 模型文件可能未包含在公开仓库中，请确保 `public/resources/live2d` 下有合法的模型文件。
*   **API 配置**: 如果涉及 AI 或云同步功能，可能需要在 `src/services/api.js` 或环境变量中配置后端服务器地址。

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证开源。详情请参阅 LICENSE 文件。
本项目为清华大学软件学院课程项目。
