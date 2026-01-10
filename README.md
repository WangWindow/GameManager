
# GameManager

GameManager 是一个基于 Tauri + Vue3 + TypeScript 的桌面游戏管理器，支持 RPG Maker MV/MZ 游戏的导入、启动和管理。项目采用 Vite 构建，界面美观，支持多平台运行。

## 主要功能

- 游戏库管理：自动扫描和导入本地 RPG Maker 游戏，展示游戏封面、版本、引擎类型等信息。
- 游戏启动：一键启动已导入的游戏，支持参数配置。
- 游戏设置：可修改游戏标题、路径、封面等信息，支持删除和路径校验。
- 应用设置：切换状态栏显示、容器清理、NW.js 下载等高级功能。
- 窗口控制：支持最小化、最大化、关闭窗口，适配 Tauri 框架。
- 数据持久化：游戏信息和设置通过 SQLite 数据库存储，后端由 Rust 实现。

## 技术栈

- 前端：Vue 3, TypeScript, TailwindCSS, Vite
- 后端：Rust, Tauri, SQLx
- 依赖：@tauri-apps/api, @iconify/vue ...

## 截图

![主界面截图](assets/screenshot-2026-01-10%2015-09-42.png)

## 快速开始

1. 安装依赖

	```bash
	npm install
	```

2. 开发模式启动

	```bash
	npm run tauri dev
	```

3. 构建发布版

	```bash
	npm run build
	npm run tauri build
	```

## 推荐开发环境

- VS Code + Volar + Tauri + rust-analyzer

---
