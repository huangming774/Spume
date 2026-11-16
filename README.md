# 桃之夭夭

一个自己从零搭起来的个人网站（）。站里放着我的文章、建站日志和一些作品展示，还有一面留言板、一个能聊天能画图的 AI 助手，以及一间可以拖动着看的 3D 星空馆。这里说说它背后用到的技术。

以前这个站是几个独立小站拼起来的：博客一套系统，建站日志另一套，各写各的，也各部署各的，改一个链接要动三个地方。这次索性整个重写了一遍，合成一个页面，共用一套后台。

页面用 React 写，配 Vite 构建。打开一次之后，在「文章」「建站日志」「星空馆」之间切换是瞬间的，不用整页重载。样式是 Tailwind 加一层自己写的 CSS，动效交给 GSAP 和 Motion，星空馆那个可以拖动的 3D 展厅用 Three.js 做。文章以 Markdown 的形式存在仓库里，排版和代码高亮在浏览器里完成，发布新文章就是往仓库提交一次。

后台接口用 Rust 写，框架叫 Axum。选 Rust 是因为它跑得快、省内存，而一个人维护的小站，我不想半夜被内存告警叫醒。文章、留言、账号这些数据存在 PostgreSQL 里，图片和 AI 生成的作品上传到 Cloudflare 的对象存储，走 CDN 分发，所以图片加载很快。

站内搜索没用现成方案，接的是支持中文分词的全文检索，搜「机器学习」能真的找到相关内容，而不是靠关键词硬凑。AI 部分能聊天也能画图，模型来自外部服务，后端做了一层调度：哪个可用就用哪个，被限流或下线的会自动绕开。

部署在 Vercel 上，推一次代码就自动构建上线。本地、容器、线上走的是同一套构建脚本，跑出来的东西没有差别。定时任务负责更新番剧、投稿这类会自己变的内容，有人留言我会收到邮件。

一个人维护一个小站，选技术只有一条标准：能让我少操心。Rust 让我别管性能，Vercel 让我别管服务器，托管数据库让我别管备份。省下来的时间，拿来写点东西。

---

以下为技术选型明细。

## 技术栈概览

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
