# 简易留言板 - Rust 版本

这是一个使用 Rust 重写的简易留言板后端，完全兼容原 Node.js 版本的 SQLite 数据库。

## 技术栈

- **Web 框架**: Actix-web 4
- **数据库**: SQLx 0.7 + SQLite
- **模板**: 直接字符串拼接（兼容原有前端）

## 功能特性

- 留言的增删查改
- 标签系统
- 留言回复
- 搜索功能
- 数据看板
- API 接口
- 完全兼容原 Node.js 版本的数据库

## 快速开始

### 1. 复制原数据库（可选）

如果你有原 Node.js 版本的数据库，可以复制过来：

```bash
cp /path/to/original/data/messages.db ./data/
```

### 2. 运行

```bash
cargo run
```

服务器将在 http://127.0.0.1:13478 启动（与原 Node.js 版本端口一致）。

### 3. 构建生产版本

```bash
cargo build --release
```

可执行文件位于 `target/release/message_board`。

## API 接口

| 路由 | 方法 | 描述 |
|------|------|------|
| `/` | GET | 首页，显示留言列表 |
| `/submit` | POST | 提交新留言 |
| `/delete` | POST | 删除留言 |
| `/reply` | POST | 提交回复 |
| `/delete-reply` | POST | 删除回复 |
| `/dashboard` | GET | 数据看板 |
| `/api/messages` | GET | 获取留言列表 JSON |
| `/api/tags` | GET | 获取标签列表 JSON |
| `/static/*` | GET | 静态资源 |

## 配置

通过环境变量配置：

- `DATABASE_URL`: SQLite 数据库连接字符串（默认：`sqlite:data/messages.db?mode=rwc`）
- `PORT`: 服务器端口（默认：`13478`）

## 与原版本的兼容性

- 数据库结构完全兼容
- 前端静态资源无需修改
- API 接口完全兼容

## 性能优势

相比 Node.js 版本，Rust 版本具有：
- 更低的内存占用
- 更高的并发处理能力
- 更快的响应速度
