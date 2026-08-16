# GameManager

GameManager 是一个使用 Rust 和 Iced 构建的原生桌面游戏管理器。

## 主要功能

- 游戏库管理：导入、扫描、搜索和管理本地游戏。
- 启动方式：支持 native、Bottles、NW.js、mkxp-z 和 external。
- 运行时管理：导入 mkxp-z，管理 NW.js 与 mkxp-z 运行时。

## 技术栈

- Rust 2024
- Iced 0.14
- Toasty 0.10 + SQLite
- NW.js、mkxp-z 和 Bottles 集成

## 快速开始

1. 安装 Rust stable

2. 开发模式启动

 ```bash
 cargo run -p gamemanager-desktop
 ```

1. 构建发布版

 ```bash
 cargo build --release -p gamemanager-desktop
 ```

## 推荐开发环境

- VS Code + rust-analyzer

---
