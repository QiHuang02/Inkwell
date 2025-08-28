# InkwellBlog

一个用 Rust 和 Axum 构建的简单博客 API。

## 🚀 快速开始

### 环境配置

1. **复制环境变量模板**：
   ```bash
   cp .env.example .env
   ```

2. **配置环境变量**：
   编辑 `.env` 文件，特别是以下重要配置：

   ```bash
   # 生成强随机 JWT 密钥
   openssl rand -base64 64
   ```

   将生成的密钥填入 `.env` 文件的 `JWT_SECRET` 字段。

3. **安装依赖并运行**：
   ```bash
   cargo run
   ```

### 环境变量说明

| 变量名                   | 说明           | 默认值              |
|-----------------------|--------------|------------------|
| `DATABASE_URL`        | 数据库连接字符串     | `sqlite:blog.db` |
| `JWT_SECRET`          | JWT 签名密钥     | -                |
| `SERVER_HOST`         | 服务器监听地址      | `127.0.0.1`      |
| `SERVER_PORT`         | 服务器监听端口      | `3000`           |
| `DB_MAX_CONNECTIONS`  | 数据库最大连接数     | `10`             |
| `JWT_EXPIRATION_DAYS` | JWT 令牌有效期（天） | `1`              |

## 📝 API 文档

启动服务后，访问 `http://localhost:3000/swagger-ui` 查看 API 文档。

## 🔐 认证系统

### 需要认证的操作

以下操作需要在请求头中包含有效的 JWT 令牌：

- **POST** `/posts` - 创建文章
- **PUT** `/posts/{id}` - 更新文章
- **DELETE** `/posts/{id}` - 删除文章
- **POST** `/posts/{id}/comments` - 创建评论
- **PUT** `/posts/{post_id}/comments/{comment_id}` - 更新评论
- **DELETE** `/posts/{post_id}/comments/{comment_id}` - 删除评论

### 获取访问令牌

1. **注册新用户**：
   ```bash
   curl -X POST http://localhost:3000/register \
     -H "Content-Type: application/json" \
     -d '{"username": "testuser", "password": "password123"}'
   ```

2. **登录获取令牌**：
   ```bash
   curl -X POST http://localhost:3000/login \
     -H "Content-Type: application/json" \
     -d '{"username": "testuser", "password": "password123"}'
   ```

3. **使用令牌访问受保护的端点**：
   ```bash
   curl -X POST http://localhost:3000/posts \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer YOUR_JWT_TOKEN_HERE" \
     -d '{"title": "我的文章", "content": "文章内容", "tags": "标签", "copyright": "版权信息"}'
   ```

   **注意**: `author` 字段已从创建文章的请求体中移除，作者信息将自动从 JWT 令牌中提取。

## 🔒 安全提醒

- **永远不要**将 `.env` 文件提交到 git 仓库
- **JWT_SECRET** 必须是强随机密钥，建议使用 `openssl rand -base64 64` 生成
- 生产环境中请使用环境变量或安全的密钥管理服务

## 🛠 开发

```bash
# 检查代码
cargo check

# 运行测试
# cargo test

# 代码格式化
# cargo fmt

# 代码检查
# cargo clippy
```
