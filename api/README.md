# API

`api/` 是一个 Axum 服务。它读取环境变量，可选连接 Postgres，然后在一个进程里提供下面这些 HTTP 接口。

没有 `DATABASE_URL` 时进程仍能启动，需要落库的接口会返回数据库未连接。媒体、模型和定时任务也都按对应环境变量是否配齐来决定能不能用。

## 运行

要求 Rust 2021 工具链。

```text
cd api
cargo run
```

默认监听 `0.0.0.0:8787`。用 `ACG_API_ADDR` 修改地址，例如 `:8787`。示例变量在 `.env.example`。

```text
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## 配置

| 变量 | 作用 |
| --- | --- |
| `DATABASE_URL` | Postgres。连接后执行 `migrations/001_init.sql` |
| `SESSION_SECRET` | 会话 cookie 签名 |
| `OWNER_EMAIL` | 站长账号 |
| `ACG_ALLOWED_ORIGINS` | 允许带 cookie 的跨域来源 |
| `DEEPSEEK_API_KEY` | 聊天模型。请求发到 `DEEPSEEK_BASE_URL/v1/chat/completions` |
| `DASHSCOPE_API_KEY` | 生图 |
| `R2_ACCOUNT_ID`、`R2_ACCESS_KEY_ID`、`R2_SECRET_ACCESS_KEY`、`R2_BUCKET`、`R2_PUBLIC_BASE` | 图片对象存储 |
| `UPSTASH_REDIS_REST_URL`、`UPSTASH_REDIS_REST_TOKEN` | 配置后，限流计数走 Redis |
| `UPSTASH_VECTOR_REST_URL`、`UPSTASH_VECTOR_REST_TOKEN` | 站内内容检索 |
| `QSTASH_CURRENT_SIGNING_KEY`、`QSTASH_NEXT_SIGNING_KEY` | 校验内部调度请求 |
| `GITHUB_TOKEN`、`GITHUB_REPO`、`OWNER_PUBLISH_GITHUB_BRANCH` | 站长把文章、友链、动态和图册写回仓库 |
| `SMTP_HOST`、`SMTP_PORT`、`SMTP_USER`、`SMTP_PASS`、`SMTP_FROM`、`MAIL_NOTIFY_TO` | 留言邮件 |
| `SYNC_TRIGGER_TOKEN` | 手动同步口令。留空时只接受站长会话 |
| `GUESTBOOK_MAIN_POSTS_ENABLED` | 主留言区是否接受新留言。友链分区不受影响 |
| `ACG_CONTENT_ROOT` | 建站索引时读取的内容目录 |
| `ACG_PUBLIC_ORIGIN` | 调度签名里核对的公开地址 |

## 接口

请求体上限 8MB。跨域在配置了来源时允许携带 cookie。

### 状态与访客

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/health` | 数据库、Redis、向量库和对象存储的探测结果 |
| GET | `/api/server/info` | 主机名以外的运行信息：系统、内存、CPU、启动时长 |
| GET | `/api/client/network` | 访客地区和掩码后的 IP |

### 账号

Cookie 名是 `acg_session`，HttpOnly，SameSite=Lax。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/auth/me` | 当前登录状态 |
| POST | `/api/auth/register` | 注册 |
| POST | `/api/auth/login` | 登录。站长在配置了安全答案时先返回校验题 |
| POST | `/api/auth/verify-security` | 提交安全答案后再发会话 |
| POST | `/api/auth/logout` | 退出 |
| PATCH | `/api/auth/profile` | 修改资料 |
| POST | `/api/auth/avatar` | 上传头像 |

### 留言

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/guestbook/messages` | 分页列表。站长能看到隐藏留言和掩码 IP |
| POST | `/api/guestbook/messages` | 发表或回复。分区为 `guestbook` 或 `friends` |
| PATCH | `/api/guestbook/messages/{id}` | 站长修改状态 |
| DELETE | `/api/guestbook/messages/{id}` | 站长删除 |

游客每天 10 条、每小时 3 条；登录用户每天 30 条、每小时 10 条。60 秒内相同正文会被拒绝。

### 内容

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/bangumi/list` | 番剧列表 |
| GET | `/api/v1/radar/feed` | UP 主动态 |
| GET | `/api/v1/bili/videos` | 投稿列表。封面走 `/api/v1/acg/image/{name}` |
| GET | `/api/v1/wallpapers/draw` | 抽一张壁纸。`source=api` 时只抽接口壁纸 |
| GET | `/api/v1/search` | 向量检索。未配置时返回空列表 |

`POST /api/v1/sync/trigger` 由部署口令或站长会话触发，立即返回。同一时间只跑一次，内容是投稿、番剧和雷达。

### 聊天与生图

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET、POST | `/api/chat` | 查询额度或发送一句问题。回复字段是 `reply` |
| GET | `/api/chat/stats` | 按天和当天小时汇总 |
| GET、POST | `/api/ai/image` | 生图额度或生成一张图 |
| GET | `/api/ai/image/gallery` | 已生成图片 |

聊天额度：游客每天 10 次，登录用户每天 50 次，两次请求至少间隔 5 秒。站长不占每日次数，仍受这 5 秒限制。另外还有每分钟次数限制：聊天 20 次，生图 6 次，图库 60 次。

聊天请求可以带上 `pageUrl`、`pageTitle` 和 `pageContext`。这些字段会放进发给模型的用户消息，固定答案仍按问题原文匹配。

### 站长

这些接口都要站长会话。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/owner/status` | 用户数、草稿、固定答案和未读提醒 |
| GET | `/api/owner/emails` | 注册用户和留言联系邮箱 |
| GET、POST | `/api/owner/drafts` | 草稿列表和新建 |
| POST | `/api/owner/publish` | 把文章写到 GitHub |
| POST | `/api/owner/friends` | 友链，写到新分支并开拉取请求 |
| POST | `/api/owner/moments` | 动态 |
| POST | `/api/owner/gallery/images` | 图册 |
| POST | `/api/owner/ai/fixed-answers` | 固定答案 |
| PATCH | `/api/owner/notifications/{id}/read` | 提醒标为已读 |
| POST | `/api/owner/uploads` | 上传图片 |
| POST | `/api/owner/uploads/presign` | 预签名上传地址 |
| POST | `/api/owner/assets` | 按文章或相册保存图片 |

### 内部任务

`/api/internal/` 下的接口在配置了 QStash 签名密钥时校验 `Upstash-Signature`。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/internal/sync/bili` | 同步投稿和番剧，等待结果 |
| POST | `/api/internal/sync/radar` | 同步雷达 |
| POST | `/api/internal/sync/wallpaper` | 从壁纸接口补图 |
| POST | `/api/internal/maintenance` | 清理过期会话和旧的小时统计 |
| POST | `/api/internal/index-content` | 把文章和日志写入向量库 |
| POST | `/api/internal/jobs/image/{id}` | 补存一张已经生成的图片 |
