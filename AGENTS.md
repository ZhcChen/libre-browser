# AGENTS.md

# 🚨 强制规则：永远使用简体中文

**核心要求：** 所有问题回复必须使用简体中文，技术术语/专有名词除外
**示例：** 使用"函数"而非"function"，使用"变量"而非"variable"

---

## 📁 Libre Browser 项目信息

**项目类型：** 基于 Tauri 的跨平台指纹浏览器
**开发语言：** Rust (后端) + Vue 3 + TypeScript (前端)
**目标：** 创建免费的、无限制的指纹浏览器，替代收费的商业产品

### 🏗️ 项目架构

```
libre-browser/
├── src-tauri/           # Rust 后端代码
│   ├── src/
│   │   └── lib.rs      # 主要 Tauri 应用逻辑和命令
│   ├── Cargo.toml      # Rust 依赖配置
│   ├── tauri.conf.json # Tauri 应用配置
├── src/
│   └── index.html      # 主界面 HTML
├── public/
│   └── app.js          # 前端逻辑
└── package.json        # 前端依赖（如果需要）
```

### 🔧 核心功能模块

1. **窗口管理器** (`create_browser_window`、`list_windows`、`close_window`)
   - 无限制创建浏览器窗口
   - 窗口位置和大小管理
   - 窗口生命周期管理

2. **指纹生成器** (`generate_fingerprint`)
   - User-Agent 伪装
   - Canvas 指纹生成
   - WebGL 指纹生成
   - 随机视窗大小

3. **代理管理** (计划中)
   - HTTP/SOCKS5 代理支持
   - 代理池管理

### 📋 开发命令

```bash
# 开发模式运行
pnpm run tauri dev

# 构建生产版本
pnpm run tauri build

# 仅构建 Rust 代码
cd src-tauri && cargo build

# 代码格式化
pnpm format

# 检查代码格式
pnpm format:check
```

### 📋 开发任务清单

详细的开发任务请参见 [`DEVELOPMENT_TASKS.md`](./work_doc/DEVELOPMENT_TASKS.md) 文件

---

# 🐍 Python 测试规则：使用 Python 3.14 无 GIL 版本

**要求：** 所有测试或测试用例使用 Python 3.14 无 GIL 版本
**工具：** 使用 `uv run --python 3.14t python` 运行
**目录：** 测试代码必须在项目内的 `tests/` 目录，避免与项目代码混淆
**注释：** 所有测试代码必须有详细的中文注释，说明测试目的和逻辑

## 🧪 测试代码规范

### 目录结构：
```
project/
├── src/           # 项目源代码
├── tests/         # 测试代码（独立目录）
└── README.md
```

### 测试文件要求：
- **位置：** 必须放在 `tests/` 目录下
- **命名：** 使用 `test_*.py` 或 `*_test.py` 格式
- **注释：** 每个函数、类、重要代码块都需要详细中文注释
- **说明：** 注释要包含测试目的、预期结果、测试逻辑

### 示例注释规范：
```python
def test_api_response():
    """测试 API 响应格式是否正确
    
    目的：验证 API 返回的数据结构符合预期格式
    输入：模拟的请求参数
    预期：返回包含 status、data、message 字段的字典
    """
    pass
```

---

# Context7 MCP 文档获取指南

## 🚨 核心规则
**📋 优先级：Context7 > Web搜索 > 本地代码库**
**🎯 适用：** 库/框架文档 | API参考 | 代码示例

---

## ⚡ 快速使用流程

1. **解析库 ID** → `resolve-library-id` 工具
2. **获取文档** → `get-library-docs` 工具 + 解析的 ID
3. **指定主题** (可选) → 如 "hooks"、"routing"、"components"

---

## 📚 使用场景

### ✅ 适用 Context7：
- **库/框架文档** - API参考、语法说明
- **代码示例** - 实际用法和最佳实践  
- **版本信息** - 最新的特性和变更
- **技术规范** - 详细的技术文档

### ❌ 不适用 Context7：
- 新闻资讯、教程博客
- GitHub issues、Stack Overflow
- 项目特定的实现代码

---

## 🎯 支持的库类型

**前端框架：** React、Next.js、Vue、Angular  
**后端框架：** Express、FastAPI、Django、Spring  
**数据库：** MongoDB、PostgreSQL、MySQL  
**UI库：** Material UI、Tailwind CSS、Ant Design  
**开发工具：** Webpack、Vite、ESLint、Babel  

---

## 💡 使用优势

🔄 **最新性** - 实时更新的官方文档  
🎯 **高质量** - 精选的权威内容和代码示例  
⚡ **高效率** - 比网页搜索更快更精准  
🎨 **易读性** - 格式化的文档展示  

---

## ⚠️ 注意事项

- 需要先解析库 ID 再获取文档
- 支持主题筛选，提高搜索精度
- 适用于大多数主流库和框架
