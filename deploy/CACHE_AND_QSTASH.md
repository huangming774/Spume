# Redis 共享缓存 + QStash 定时任务（第一期）

两者都是**可选增强**：不配置时后端行为与原来完全一致（进程内缓存 / 手动触发同步）。

## 一、Redis 共享缓存（Upstash Redis REST）

### 解决什么
Vercel 多实例 + 冷启动：进程内缓存一冷就空、实例间不共享。配了 Redis 后热点读走共享缓存。

### 缓存清单（key 前缀 `cache:`）
| key | TTL | 失效时机 |
| --- | --- | --- |
| `music:pl:{id}` | 600s / 失败 30s | 后台保存歌单 |
| `deploy:status` | 60s / 失败 30s | 自然过期 |
| `site:profile` | 60s | 上传头像 / 改昵称 |
| `content:{kind}`、`content:recent` | 30s | 发布 / 删除条目 |
| `weather:{ip}` | 600s / 失败 30s | 自然过期 |
| `geo:{ip}` | 3600s | 自然过期 |
| `health:snapshot` | 15s | 后台周期刷新覆写 |

读取层级统一为：进程内 → Redis → 上游/DB；写路径主动 DEL，避免脏缓存。
实现见 `api/_core/src/cache.rs`（REST pipeline，1.5s 超时，出错静默降级）。

### 启用
Vercel 环境变量（Upstash 控制台 → Redis → REST API）：
```
UPSTASH_REDIS_REST_URL=https://<id>.upstash.io
UPSTASH_REDIS_REST_TOKEN=<token>
```
健康接口 `redis` 字段变为 `up` 即生效。注意免费档每日命令数上限，TTL 已按低频读设计。

## 二、QStash 定时任务

### 解决什么
serverless 不能常驻 cron。QStash 按 cron 签名回调内部接口，代替后台 worker。

### 调度表（UTC）
| cron | 目标 | 作用 |
| --- | --- | --- |
| `0 * * * *` | `/api/v1/sync/trigger` | 每小时全量同步（bili 等） |
| `*/30 * * * *` | `/api/internal/sync/radar` | 雷达流每 30 分钟 |
| `0 3 * * *` | `/api/internal/maintenance` | 每日维护 |
| `20 3 * * *` | `/api/internal/index-content` | 每日重建向量索引 |
| `40 3 * * *` | `/api/internal/refresh-ai-model` | 每日轮换免费模型 |
| `0 4 * * *` | `/api/internal/sync/wallpaper` | 每日壁纸池同步 |

### 启用
1. Vercel 环境变量：
   ```
   QSTASH_CURRENT_SIGNING_KEY=<Upstash QStash 当前签名 key>
   QSTASH_NEXT_SIGNING_KEY=<下一签名 key>
   ACG_PUBLIC_ORIGIN=https://www.miku.coffee
   ```
2. 本机执行（幂等，重复运行不会建重复调度）：
   ```powershell
   $env:QSTASH_TOKEN = "<QStash token>"
   ./deploy/qstash-schedules.ps1
   ```
3. `/api/v1/sync/trigger` 已支持 `Upstash-Signature` 签名分支；人工触发（令牌/站长会话）不受影响。

### 安全
内部接口仍走 `qstash.rs` 签名校验（含目标 URL、时间窗、body 哈希）；未配签名 key 时内部接口默认拒绝（除 `ACG_ALLOW_UNSIGNED_INTERNAL=true` 的本地开发模式）。

## 三、第二期：发布异步化 + Redis 新能力

### 发布异步化（3 件）
发布先落库立即返回，以下三件异步做，配了 `QSTASH_TOKEN` 走 QStash（重试 3 次），否则进程内 spawn 尽力交付：
1. **GitHub 写回**：`/api/internal/publish`。文章写 md、友链直写主分支、碎语/相册写数据文件。
2. **向量索引重建**：`/api/internal/reindex`（发布后触发，另有每日定时）。
3. **邮件通知**：`/api/internal/mail`。留言/友链申请/站长回复的邮件改异步发送。

启用：Vercel 加 `QSTASH_TOKEN`（与签名 key 同一 QStash 项目）。不配时行为回退同步/spawn，不丢功能。

### Redis 新能力
| 能力 | key | 说明 |
| --- | --- | --- |
| 浏览计数/热门 | `views:{kind}:{ISO周}` | 文章页上报 `POST /api/client/view`、壁纸抽取自动 ZINCRBY；`GET /api/client/hot?kind=article` 出周榜 Top6，首页「本周热门」条消费。不写 DB。 |
| 配额计数 | `quota:chat:{key}:{date}`、`quota:image:user:{id}:{date}` | 聊天/生图每日配额改 INCR+TTL(26h)，5 秒间隔用 `pace:*` SETNX；失败 DECR 回退。未配 Redis 回退 DB 原子 UPSERT。 |
| 短窗去重 | `dedupe:gb:{ip}:{content}` | 留言提交 SETNX 60s 防连点；未配/出错回退 DB 去重查询。 |

注意：配额与去重的 Redis 路径以 Redis 为权威计数；从「未配」切到「已配」当天计数从 0 起（一次性宽松），次日自然对齐。
