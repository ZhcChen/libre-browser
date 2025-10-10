# Libre Browser

🚀 基于 Tauri 的跨平台指纹浏览器 - 免费且无限制的浏览器指纹管理工具

## 📋 项目概述

Libre Browser 是一个开源的指纹浏览器项目，旨在为用户提供免费、无限制的浏览器指纹伪装和管理功能。作为商业指纹浏览器的替代方案，本项目支持多窗口管理、指纹伪装、代理集成等核心功能。

### 🎯 核心特性

- ✨ **无窗口限制** - 支持创建无限数量的浏览器窗口
- 🎭 **指纹伪装** - User-Agent、Canvas、WebGL 等多维度指纹随机化
- 🌐 **代理支持** - HTTP/SOCKS5 代理集成与管理
- 🔒 **数据隔离** - 独立的 Cookie 存储和用户配置
- 🎨 **现代界面** - 基于 Vue 3 + TypeScript 的现代化前端
- 🔄 **跨平台** - 支持 Windows、macOS、Linux

## 🏗️ 技术架构

### 技术栈
- **后端**: Rust + Tauri 框架
- **前端**: Vue 3 + TypeScript
- **构建工具**: Vite
- **代码规范**: Prettier

### 项目结构
```
libre-browser/
├── src-tauri/           # Rust 后端代码
│   ├── src/
│   │   └── lib.rs      # 主要 Tauri 应用逻辑和命令
│   ├── Cargo.toml      # Rust 依赖配置
│   └── tauri.conf.json # Tauri 应用配置
├── src/
│   ├── main.ts         # Vue 应用入口
│   ├── App.vue         # 主应用组件
│   └── components/     # Vue 组件目录
├── public/             # 静态资源
├── package.json        # 前端依赖配置
└── prettier.config.js  # 代码格式化配置
```

### 核心功能模块

1. **窗口管理器**
   - 无限制创建浏览器窗口
   - 窗口位置和大小管理
   - 窗口生命周期管理

2. **指纹生成器**
   - User-Agent 伪装
   - Canvas 指纹生成
   - WebGL 指纹生成
   - 随机视窗大小

3. **代理管理**
   - HTTP/SOCKS5 代理支持
   - 代理池管理

## 🚀 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.77
- pnpm (推荐) 或 npm

### 安装依赖
```bash
pnpm install
```

### 开发模式
```bash
pnpm run tauri dev
```

### 代码格式化
```bash
pnpm format
# 或者检查格式
pnpm format:check
```

## 📦 构建与打包

### 构建前端
```bash
pnpm build
```

### 构建完整应用
```bash
pnpm run tauri build
```

### 仅构建 Rust 代码
```bash
cd src-tauri && cargo build
```

构建完成后，可执行文件将位于：
- **Windows**: `src-tauri/target/release/libre-browser.exe`
- **macOS**: `src-tauri/target/release/bundle/macos/Libre Browser.app`
- **Linux**: `src-tauri/target/release/libre-browser`

## 🛠️ 开发指南

### IDE 推荐
- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 开发任务
详细的开发任务清单请参考 [`DEVELOPMENT_TASKS.md`](./DEVELOPMENT_TASKS.md)

### 开发规范
- 所有代码使用 Prettier 进行格式化
- 遵循项目架构和模块划分
- 参考项目记忆文件 [`AGENTS.md`](./AGENTS.md)

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🤝 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进项目！

## 🔗 相关链接

- [Tauri 文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [TypeScript 文档](https://www.typescriptlang.org/)
