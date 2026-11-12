# Rust 后端（api/）安全与性能审查报告

审查时间：2026-11-12。范围：`api/src` 24 个源文件 + `api/migrations/001_init.sql`（约 1 万行），旧 Go 后端 acg-api 不在范围内。方法：5 个并行审查智能体按模块分工（入口/鉴权/输入处理/存储/外部集成），高危结论经人工逐条抽查核实。

总体评价：代码底子好——全部参数化 SQL 无注入面、QStash 验签（HS256 强制 + exp/nbf + 恒时比较）扎实、reqwest Client 经 AppState 复用、内部任务有签名鉴权、时间戳统一 TIMESTAMPTZ。主要短板集中在：错误信息直出、站长注册流程、Supabase 池化兼容、以及 serverless 场景下的串行 DB 往返与冷启动 DDL。

---

## 高危（4 项，上线前必须修）

| # | 问题 | 位置 | 修复 |
|---|---|---|---|
| H1 | **错误响应泄漏内部细节**：`query_failed` 把 sqlx/reqwest/SMTP 错误原文拼进 message 返回客户端，可暴露 SQL、连接地址、上游 URL。共 9 处 | routes.rs:118-124（另 441、495、839、1264、2222、2644、3527、3593） | 统一固定文案，原始 error 仅写 tracing |
| H2 | **站长账号可抢注**：注册时 `email == OWNER_EMAIL` 即授 owner；`OWNER_SECURITY_ANSWER` 默认空会跳过安全问答直接发 owner 会话，且无邮箱所有权验证 | routes.rs:276-291、config.rs:56 | 强制安全问答非空，或邮件验证通过后才发 owner 会话 |
| H3 | **事务池化与预处理语句冲突**：sqlx 默认缓存 prepared statement，与 Supabase 6543 事务池化不兼容，线上会间歇报 "prepared statement" 错误 | db.rs:6 | `PgConnectOptions` 设 `statement_cache_capacity(0)`，或改 5432 会话模式 |
| H4 | **壁纸同步可被匿名请求放大**：图池为空时每个 `apiOnly=1` 请求内联 `sync_pool`，顺序连打上游最多 24 次（单次 15s 超时），上游故障期间形成放大 | wallpaper.rs:86（触发点 routes.rs:1227） | sync_pool 加进程内互斥 + 60s 冷却，或改 QStash 后台任务 |

## 中危（18 项，尽快修）

| # | 问题 | 位置 | 修复 |
|---|---|---|---|
| M1 | CORS 解析失败/allowed_origins 为空时回落 `AllowOrigin::any()` 且 `allow_credentials(true)`，等于配置错误即全放行 | routes.rs:90-97 | origins 为空直接启动失败，禁止回落 any() |
| M2 | 限流身份可伪造：`client_ip` 取 X-Forwarded-For 最左值，伪造头即可绕过 chat/生图/留言限流与去重 | routes.rs:2308-2319 | 只信平台注入的 x-real-ip 或链路最右地址 |
| M3 | bcrypt（cost 12）在 async fn 里同步执行，每次 hash/verify 占住 tokio worker 数百毫秒 | routes.rs:266、320 | 移入 `spawn_blocking` |
| M4 | 留言 XSS 黑名单可绕过：无 `>` 的标签直接放行、事件黑名单仅 13 项（缺 onpointerdown/onwheel 等），`<img src=x onwheel=…>` 类载荷可通过 | guestbook.rs:709、126 | 改为输出侧统一转义/白名单清洗，勿依赖黑名单 |
| M5 | 聊天日配额读-判-增非原子：`used` 先读后 +1，模型耗时 >5s 时并发请求均读旧值可超每日限额 | routes.rs:1534、1750 | 在 claim_chat_turn 的 CTE 里加 `used < limit` 原子判定并累加 |
| M6 | 一次 chat 请求串行约 6 次远程 DB 往返（session+quota+claim+record×2+bump），record_chat 两个 insert 未合并 | routes.rs:1542-1632、1794-1811 | 合并为 CTE/单事务，可并行的并行 |
| M7 | 冷启动全量执行 DDL：每个 serverless 实例启动按分号拆分执行 40+ 条语句，拆分脆弱、多实例并发有竞态、拖慢首请求 | db.rs:7-13、lib.rs:47-51 | 改 `sqlx::migrate!`（带 advisory lock）只跑增量 |
| M8 | 连接池配置过简：仅 `max_connections(5)`，无 acquire_timeout/idle_timeout，Vercel 多实例并发易耗尽 Supabase 连接数 | db.rs:6 | serverless 降到 1~2 并设 acquire_timeout |
| M9 | 预签名 PUT 无范围约束：UNSIGNED-PAYLOAD、仅签 host，不限 Content-Type/大小/key 前缀；key 由请求方任意指定，持 URL 者可覆盖桶内任意对象 | r2.rs:17-31、routes.rs:3139 | key 强制 `uploads/` 前缀 + 字符白名单，签名绑定 Content-Type |
| M10 | 图片下载整读进内存：`bytes().await` 全量缓冲后才校验 8MiB 上限，异常大响应可打爆内存 | image_store.rs:73-79 | 预查 Content-Length，流式限长中止 |
| M11 | 索引与查询脱节：留言主查询按 `(channel, parent_id)` 过滤 + `ORDER BY id DESC`，现有 `(channel, created_at)` 索引用不上；限流计数查询全表扫 | 001_init.sql:36、guestbook.rs:380、routes.rs:905-968 | 补 `(channel,parent_id,id)`、`(user_id,created_at)`、`(ip_hash,ip_user_agent_hash)` 索引 |
| M12 | GitHub 发布请求无超时：共享 Client 默认无 timeout，GitHub 挂起会占住发布请求直到平台强断 | github_publish.rs:603 | `send()` 加 `.timeout(10~15s)` |
| M13 | B站投稿全量同步无节流无增量：顺序拉最多 100 页全量重放，页间无延迟，易触发风控 | bili.rs:190-213 | 页间加间隔，按 synced_at 增量拉取 |
| M14 | 追番分页截断：`pn=1&ps=30` 写死，关注超 30 部时静默丢弃且整表替换缓存 | feed.rs:196 | 按 total 循环翻页取全量再替换 |
| M15 | 向量索引重建非原子：先 delete_prefix 再 upsert，中途失败使搜索空窗至下次任务成功 | jobs.rs:90-95 | 先写新前缀再删旧的，或版本化前缀切换 |
| M16 | 提示词上下文未与指令隔离：页面上下文字段与用户问题拼进同一条 user 消息，仅用明文伪分隔符，伪造分隔符可改写指令 | chat_prompt.rs:68-82 | 系统提示声明上下文为不可信数据，字段 JSON 编码定界 |
| M17 | IP/UA 指纹无盐哈希：SHA-256 截 16 字节，IPv4 空间 2^32 可枚举反查 | audit.rs:66-76 | 改 HMAC-SHA256(服务端密钥, 值) |
| M18 | YAML 标量引号判定不全：标题含 `": "` 或以 `-`/`*`/`&` 等开头时原样输出，产出非法 front matter | post_markdown.rs:160-174 | 无条件走双引号转义输出 |

## 低危（16 项，顺手修）

| # | 问题 | 位置 |
|---|---|---|
| L1 | 会话 Cookie `AUTH_COOKIE_SECURE` 默认 false，生产应默认 true（Vercel 全 HTTPS） | config.rs:57、routes.rs:181-187 |
| L2 | `Config` 派生 Debug 含全部密钥（R2/GitHub/SMTP/QStash），任一 `{:?}` 日志即泄漏 | config.rs:1-2、29 |
| L3 | 缺安全响应头（X-Content-Type-Options/HSTS/frame-ancestors/Referrer-Policy） | routes.rs:18-80 |
| L4 | B站上游空响应（code 0 且 count=0）会清空投稿缓存并置空 owner_name | bili.rs:367 |
| L5 | 壁纸外链无域名白名单：仅 https + .webp 后缀校验 | wallpaper.rs:47-58 |
| L6 | radar 链接 bvid 校验弱于投稿侧（仅前缀+长度），可拼进 video 外链 | feed.rs:257 |
| L7 | 图片下载来源校验弱：仅 https + 响应头 image/*（含 svg，无魔数嗅探、无私网 IP 拒绝，SSRF 面） | image_store.rs:46-71 |
| L8 | 清理/排序列缺索引：sessions.expires_at、ai_images.created_at、owner_drafts.updated_at | 001_init.sql:11、103、121 |
| L9 | restock 失败无状态：QStash 重试每次全量重下载，无 attempts 计数 | jobs.rs:100-125 |
| L10 | async 中同步 IO：read_dir/read_to_string 阻塞 worker（量小） | jobs.rs:219-236 |
| L11 | QStash 有效期内（~5min）同签名请求可重放触发任务 | qstash.rs:114 |
| L12 | 跨 UTC 午夜 bump_quota 命中 0 行返回 500，已生成回复被丢弃 | routes.rs:1751、1757 |
| L13 | 额度/留言按 UTC 天重置，与东八区"每天"差 8 小时 | routes.rs:1686、902 |
| L14 | 留言分页无上界：极大 page 使 OFFSET 巨量或 i64 溢出 | guestbook.rs:365 |
| L15 | 地区查询明文 HTTP（ip-api），回包可被篡改入库 | guestbook.rs:644 |
| L16 | 文本截断按字符/字节非字素簇（ZWJ emoji 断裂）；全量折叠后再截断浪费开销；js_source 括号匹配不识别正则/模板插值、"date:" 子串计数、U+2028/29 未转义 | chat_prompt.rs:43、125、135、audit.rs:38、js_source.rs:178、503-529、570-581 |

---

## 修复优先级建议

1. **立即**：H1、H2（安全暴露）、H3（线上稳定性）。
2. **上线前**：H4、M1、M2、M4、M5、M9。
3. **性能专项**：M6（chat 6 次串行往返）、M7+M8（冷启动 DDL + 池配置）、M11（索引）。这三组对 serverless 下的 p95 延迟影响最大。
4. 其余中/低危可随迭代处理。
