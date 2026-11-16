# 技术栈概览

Spume由两部分组成：Rust 后端 `api/` 和合并后的前端 SPA `main/`。
这里只列技术选型，架构和部署细节见 [DOCKER.md](../deploy/DOCKER.md)。

## 后端 · `api/`（Rust）

**语言与运行时**：Rust（edition 2021）+ Tokio 异步运行时。

**Web 框架**：Axum 0.8，路由、JSON 提取器、multipart 文件上传、共享 `AppState` 都在它之上。

**数据与存储**：

| 用途 | 选型 |
| --- | --- |
| 主数据库 | PostgreSQL（Supabase 托管），SQLx 0.8，启动时执行 `migrations/001_init.sql` |
| 图床 / 生图产物 | Cloudflare R2 |
| 站内语义/全文搜索 | Zilliz Cloud（Milvus REST v2，BM25 全文检索 + 中文分词） |
| 限流与计数 | Upstash Redis（ZSET 滑动窗口） |
| 后台任务调度 | Upstash QStash（带签名校验的内部任务接口） |

**主要依赖**：

- `reqwest`（rustls）——调用 DeepSeek/Agnes/DashScope 等外部 AI 与内容 API
- `lettre`（SMTP over TLS）——留言邮件通知
- `bcrypt` ——账号密码哈希
- `hmac` / `sha2` / `md-5` ——B 站 WBI 签名、内部任务请求签名
- `serde` / `serde_json` ——接口序列化
- `dotenvy` ——启动时读 `/opt/acg-api/.env` 和 `.env`（不覆盖已有环境变量）
- `tracing` + `tracing-subscriber` ——结构化日志
- `thiserror` ——内部错误类型

**接入的外部 AI 服务**：DeepSeek（聊天）、Agnes（生图）、DashScope（生图备选）。

**内容发布**：站长后台通过 GitHub Contents API 把文章、友链、动态、图册写回仓库。

## 前端 · `main/`（合并 SPA）

**框架与构建**：React 18 + Vite 8，JavaScript/JSX 为主，`src/subapps/journal`（建站日志子应用）用 TypeScript/TSX。

**路由**：React Router v6。`/blog`（文章）、`/build`（建站日志）、`/projects`（星空馆）都是 SPA 路由，不再是独立子站。

**样式**：Tailwind CSS 3 + 自定义 CSS。journal 子应用的纯 CSS 用 `postcss-prefix-selector` 做作用域隔离，避免污染主站。

**动画与交互**：GSAP（配 `@gsap/react`）、Motion（`motion` 包，滚动/入场动画）。

**3D 与图标**：Three.js（星空馆 `/projects`，原生 JS 模块，React 组件包装后动态 `import()` 挂载并负责销毁清理）、`lucide` / `lucide-react` 图标。

**内容渲染**：`marked` + `highlight.js` 渲染 Markdown 文章与代码高亮。文章源是 `blog/source/_posts/*.md`，构建期由 Vite 的 `import.meta.glob` 内联进包，不再依赖 Hexo 生成 HTML；建站日志内容是构建期生成的 `content.json`。

**AI 助手**：`shared/ai-assistant/`，原生 JS + CSS 的独立组件，可挂在任意页面，走后端 `/api/chat`、`/api/ai/image`。

## 构建与部署

**统一构建入口**：`tools/vercel-build.mjs`，把 `main/dist` + `shared/ai-assistant` 组装成 `out/`，本地、CI、容器共用同一套逻辑。

**容器化**：`api/Dockerfile`（Rust 多阶段构建）+ `deploy/docker/Dockerfile.web`（Node 构建 + nginx 运行），nginx 负责 SPA fallback、`/api/` 反代和 TLS。

**CI/CD**：GitHub Actions（`.github/workflows/docker.yml`）构建镜像推 GHCR，服务器定时 `docker compose pull` 部署（见 [DOCKER.md](../deploy/DOCKER.md)）。
