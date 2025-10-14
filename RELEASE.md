# 🚀 Libre Browser 发布指南

本文档说明如何使用自动化工具构建和发布 Libre Browser 应用。

## 📋 发布流程概览

1. **开发完成** → 2. **测试验证** → 3. **版本管理** → 4. **自动构建** → 5. **发布上线**

## 🛠️ 准备工作

### 系统要求
- Rust 1.70+
- Node.js 18+
- Git

### 依赖安装
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 安装 Node.js (使用 nvm 推荐)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

### 克隆仓库
```bash
git clone <repository-url>
cd libre-browser
```

## 🎯 自动发布功能

### 方式一：使用 GitHub Actions (推荐)

当您推送版本标签到 GitHub 时，会自动触发构建和发布：

```bash
# 升级版本并推送
npm run version:patch    # 补丁版本: 0.1.0 → 0.1.1
npm run version:minor    # 次版本: 0.1.0 → 0.2.0
npm run version:major    # 主版本: 0.1.0 → 1.0.0

# 推送到远程仓库（会自动触发 GitHub Actions）
git push origin main --tags
```

### 方式二：使用本地发布脚本

```bash
# 完整发布流程（包含所有检查）
npm run release

# 分步骤发布
npm run release:test      # 仅运行测试
npm run release:build     # 仅构建项目
npm run release:bump     # 仅升级版本
npm run release:tag      # 仅创建标签
npm run release:push     # 仅推送到远程
```

## 📦 发布配置

### Tauri 配置
确保 `src-tauri/tauri.conf.json` 中配置了正确的发布设置：

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### GitHub Actions 配置
- **工作流文件**: `.github/workflows/release.yml`
- **触发条件**: 推送 `v*` 标签时
- **支持平台**: Windows, macOS, Linux
- **输出格式**: MSI, DMG, AppImage

## 🔄 版本管理

### 语义化版本控制
- **主版本 (Major)**: 不兼容的 API 修改
- **次版本 (Minor)**: 向后兼容的功能性新增
- **补丁版本 (Patch)**: 向后兼容的问题修正

### 版本升级命令
```bash
# 自动升级并打标签
npm run version:patch   # 0.1.0 → 0.1.1
npm run version:minor   # 0.1.0 → 0.1.2
npm run version:major   # 0.1.0 → 1.0.0

# 手动控制版本流程
./scripts/release.sh bump patch  # 升级版本
./scripts/release.sh tag          # 创建标签
./scripts/release.sh push         # 推送远程
```

## 🧪 质量保证

### 自动化检查
```bash
# 代码格式化检查
npm run format:check

# Rust 代码质量检查
npm run clippy

# 运行测试套件
npm run test

# 完整质量检查
npm run release:test
```

### 预发布检查清单
- [ ] 所有测试通过
- [ ] 代码格式化检查通过
- [ ] Clippy 无警告
- [ ] 应用在本地正常启动
- [ ] 版本号符合语义化规范
- [ ] CHANGELOG 已更新

## 📂 构建产物

### 本地构建
```bash
# 构建 Tauri 应用
npm run tauri:build

# 构建产物位置
ls src-tauri/target/release/bundle/
```

### 支持的安装包格式
- **Windows**: `.msi` 安装程序
- **macOS**: `.dmg` 磁盘映像
- **Linux**: `.AppImage` 便携应用

## 🚀 发布步骤详解

### 1. 开发和测试
```bash
# 启动开发环境
npm run dev

# 运行测试
npm run test

# 代码质量检查
npm run clippy
```

### 2. 版本管理
```bash
# 方式A: 快速升级（推荐）
npm run version:patch

# 方式B: 手动控制
./scripts/release.sh bump patch
./scripts/release.sh tag
```

### 3. 推送到远程仓库
```bash
# 推送代码和标签
git push origin main --tags
```

### 4. 自动化构建和发布
- GitHub Actions 会自动检测到新标签
- 运行测试套件
- 跨平台构建应用
- 自动创建 GitHub Release
- 上传构建产物到 Release

## 🔧 高级配置

### 自定义构建脚本
编辑 `scripts/release.sh` 脚本来自定义发布流程：

```bash
# 添加自定义检查步骤
echo "运行自定义检查..."
custom_check_command

# 添加自定义构建步骤
echo "执行自定义构建..."
custom_build_command
```

### 环境变量配置
创建 `.env` 文件来配置敏感信息：

```bash
# GitHub Token（用于自动发布）
GITHUB_TOKEN=your_token_here

# 自定义构建选项
TAURI_CUSTOM_ARGS="--custom-option"
```

## 🐛 故障排除

### 常见问题

**Q: 构建失败，提示依赖问题**
```bash
# 清理并重新安装依赖
npm run clean
rm -rf node_modules package-lock.json
npm install
cargo clean
```

**Q: 代码格式化检查失败**
```bash
# 自动修复格式问题
npm run format

# 检查特定文件
npm run format:check
```

**Q: GitHub Actions 构建失败**
- 检查 `.github/workflows/release.yml` 配置
- 查看 Actions 页面中的错误日志
- 确保所有依赖版本兼容

**Q: 发布后应用无法启动**
- 检查构建完整性
- 验证签名配置
- 查看应用日志

## 📚 相关文档

- [Tauri 官方文档](https://tauri.app/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [语义化版本控制](https://semver.org/)
- [项目 README](./README.md)

---

🤖 *此文档随项目更新而更新*