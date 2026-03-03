# Simple Message Board / 简易留言板

[![CI](https://github.com/chenpu17/Simple-Message-Board2/actions/workflows/ci.yml/badge.svg)](https://github.com/chenpu17/Simple-Message-Board2/actions/workflows/ci.yml)
[![Release](https://github.com/chenpu17/Simple-Message-Board2/actions/workflows/release.yml/badge.svg)](https://github.com/chenpu17/Simple-Message-Board2/actions/workflows/release.yml)
[![npm version](https://img.shields.io/npm/v/simple-message-board.svg)](https://www.npmjs.com/package/simple-message-board)
[![License](https://img.shields.io/npm/l/simple-message-board.svg)](LICENSE)

A high-performance message board backend written in Rust, fully compatible with the original [Node.js version](https://github.com/chenpu17/Simple-Message-Board).

一个高性能的留言板后端服务，使用 Rust 编写，完全兼容原有的 [Node.js 版本](https://github.com/chenpu17/Simple-Message-Board)。

---

## Features / 功能特性

### English

- 📝 **Message Management** - Create, delete, search messages with pagination
- 🏷️ **Tag System** - Organize messages with tags, filter by tag
- 💬 **Reply System** - Reply to messages, threaded discussions
- 🔍 **Full-text Search** - Search messages by content
- 📊 **Dashboard** - Statistics and analytics visualization
- 🔌 **REST API** - JSON API for integration
- 🌐 **Multi-theme Support** - Light, Dark, Cyberpunk themes
- 🚀 **High Performance** - Rust backend with low memory footprint
- 💾 **SQLite Compatible** - Works with existing Node.js database

### 中文

- 📝 **留言管理** - 创建、删除、搜索留言，支持分页
- 🏷️ **标签系统** - 使用标签组织留言，按标签筛选
- 💬 **回复系统** - 回复留言，支持讨论
- 🔍 **全文搜索** - 按内容搜索留言
- 📊 **数据看板** - 统计数据可视化
- 🔌 **REST API** - JSON 格式接口，便于集成
- 🌐 **多主题支持** - 亮色、暗色、赛博朋克主题
- 🚀 **高性能** - Rust 后端，低内存占用
- 💾 **SQLite 兼容** - 兼容现有 Node.js 数据库

---

## Installation / 安装

### Via npm (Recommended) / 通过 npm 安装（推荐）

```bash
# Install globally / 全局安装
npm install -g simple-message-board

# Run / 运行
message-board
```

**Supported Platforms / 支持平台:**
- macOS (ARM64)
- Linux (x64, ARM64)
- Windows (x64)

### From Source / 从源码构建

```bash
# Clone repository / 克隆仓库
git clone https://github.com/chenpu17/Simple-Message-Board2.git
cd Simple-Message-Board2

# Build / 构建
cargo build --release

# Run / 运行
./target/release/message_board
```

---

## Quick Start / 快速开始

### 1. Migrate from Node.js version / 从 Node.js 版本迁移

If you have an existing database from the Node.js version, the Rust version uses the **same data directory** (`~/.message-board/`), so no migration is needed!

如果你有 Node.js 版本的数据库，Rust 版本使用**相同的数据目录** (`~/.message-board/`)，无需迁移！

```bash
# Data is stored in the same location / 数据存储在相同位置
# ~/.message-board/messages.db

# Just install and run! / 直接安装运行即可！
npm install -g simple-message-board
message-board
```

### 2. Start the server / 启动服务

```bash
# Using npm / 使用 npm
message-board

# Or from source / 或从源码
cargo run
```

Server will start at http://127.0.0.1:13478

服务将在 http://127.0.0.1:13478 启动

### 3. Access the application / 访问应用

- **Home Page / 首页**: http://127.0.0.1:13478/
- **Dashboard / 数据看板**: http://127.0.0.1:13478/dashboard

---

## API Reference / API 接口

### Pages / 页面路由

| Route | Method | Description |
|-------|--------|-------------|
| `/` | GET | Home page with message list, search, pagination |
| `/dashboard` | GET | Statistics dashboard |
| `/static/*` | GET | Static files (CSS, JS, themes) |

### Form Actions / 表单操作

| Route | Method | Description |
|-------|--------|-------------|
| `/submit` | POST | Submit a new message |
| `/delete` | POST | Delete a message |
| `/reply` | POST | Submit a reply |
| `/delete-reply` | POST | Delete a reply |

### JSON API / JSON 接口

| Route | Method | Parameters | Description |
|-------|--------|------------|-------------|
| `/api/messages` | GET | `since_id`, `limit` | Get messages with details |
| `/api/tags` | GET | - | Get all tags with counts |

---

## Configuration / 配置

Environment variables / 环境变量:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:~/.message-board/messages.db?mode=rwc` | SQLite connection string |
| `DATA_DIR` | `~/.message-board` | Data directory for database storage |
| `PORT` | `13478` | Server port |

### Example / 示例

```bash
# Custom port / 自定义端口
PORT=8080 message-board

# Custom data directory / 自定义数据目录
DATA_DIR=/path/to/data message-board

# Custom database path / 自定义数据库路径
DATABASE_URL="sqlite:/data/messages.db?mode=rwc" message-board
```

---

## Tech Stack / 技术栈

- **Language / 语言**: Rust
- **Web Framework / Web 框架**: Actix-web 4
- **Database / 数据库**: SQLx 0.8 + SQLite
- **DateTime / 日期时间**: Chrono

---

## Project Structure / 项目结构

```
.
├── src/
│   ├── main.rs          # Entry point / 入口
│   ├── lib.rs           # Library exports / 库导出
│   ├── config.rs        # Configuration constants / 配置常量
│   ├── utils.rs         # Utility functions / 工具函数
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models.rs    # Data models / 数据模型
│   │   └── repository.rs # Database operations / 数据库操作
│   └── handlers/
│       ├── mod.rs
│       ├── home.rs      # Home page handler / 首页处理
│       ├── dashboard.rs # Dashboard handler / 看板处理
│       └── api.rs       # API handlers / API 处理
├── public/
│   ├── app.css          # Styles / 样式
│   ├── app.js           # Frontend JavaScript / 前端 JS
│   └── themes/          # Theme files / 主题文件
├── templates/           # HTML templates / HTML 模板
└── tests/               # Test files / 测试文件
```

---

## Development / 开发

```bash
# Run tests / 运行测试
cargo test

# Run with auto-reload (needs cargo-watch) / 自动重载运行
cargo watch -x run

# Format code / 格式化代码
cargo fmt

# Lint / 代码检查
cargo clippy
```

---

## Performance / 性能

Compared to the Node.js version / 相比 Node.js 版本:

- **Memory / 内存**: ~10-20MB vs ~50-100MB
- **Binary Size / 二进制大小**: ~15MB (single executable)
- **Startup Time / 启动时间**: < 100ms
- **Concurrency / 并发**: Native async I/O with Tokio

---

## Migration Guide / 迁移指南

### From Node.js Version / 从 Node.js 版本迁移

1. **Database / 数据库**: Fully compatible! Same data directory (`~/.message-board/`), no migration needed / 完全兼容！相同数据目录，无需迁移
2. **Static Files / 静态文件**: Included in npm package / 包含在 npm 包中
3. **Templates / 模板**: Compatible with original HTML / 兼容原有 HTML
4. **API / 接口**: 100% compatible / 100% 兼容
5. **Command / 命令**: Same command `message-board` / 相同命令 `message-board`

---

## Related Projects / 相关项目

- [Simple-Message-Board](https://github.com/chenpu17/Simple-Message-Board) - Original Node.js version / 原 Node.js 版本

---

## License / 许可证

MIT

---

## Contributing / 贡献

Issues and pull requests are welcome!

欢迎提交 Issue 和 Pull Request！
