# 博客 AI 小精灵

右下角胶囊按钮 **✦ 问问博客**，展开后与 **博客小精灵** 对话。所有对话经后端 `POST /api/chat` 转发至 **DeepSeek-v4-flash**，前端不接触 API Key。

## 前端（全站共用）

- 静态资源：`shared/ai-assistant/` → 部署到 `https://taozhiyy.top/ai-assistant/`
- 主站 `main/index.html`、`build/index.html`、博客 `_config.butterfly.yml` inject 均引用上述路径
- 博客 **勿** 再开 Butterfly 的 Tidio/Chatra（`_config.butterfly.yml` → `chat.use` 留空），避免与全站小精灵重复
- 打开面板会显示 **当前页面** 标题；发消息时附带 `pageUrl`、`pageTitle` 供模型理解上下文（不抓取整页 HTML）

配额展示：`今日剩余：7/10`（游客）或 `今日剩余：42/50`（登录）

## 后端

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/chat` | 查询今日剩余次数（不扣次） |
| POST | `/api/chat` | `{ "message": "...", "pageUrl": "...", "pageTitle": "..." }` → DeepSeek 回复 |
| GET | `/api/chat/stats?days=14` | 聚合调用统计（成功/失败/限流，不含聊天内容） |

由 `acg-api` 提供，Nginx 将 `/api/` 反代到 `127.0.0.1:8787`。

## 环境变量

服务器 `/opt/acg-api/.env`（勿提交 Git）：

```bash
DEEPSEEK_API_KEY=sk-xxx
DEEPSEEK_BASE_URL=https://api.deepseek.com
DEEPSEEK_MODEL=deepseek-v4-flash
```

重启：`sudo systemctl restart acg-api`

## 次数与频率

| 身份 | 每日上限 | 识别 |
|------|----------|------|
| 游客 | 10 | IP + User-Agent 哈希 |
| 登录 | 50 | Cookie `blog_user_id` 或 Header `X-Blog-User-Id` |

同一身份 **5 秒内最多 1 次** 请求。超限返回 429，**不调用 DeepSeek**。

## 成本控制

- 用户输入最多 **500 字**
- `max_tokens` **400**
- 简短 system prompt，不注入整篇文章
- 失败不重试

## 登录

在任意页面打开右下角 **✦ 问问博客** → 面板内 **登录 / 注册**（`/login` 会跳转并自动打开面板）。

| 身份 | 额度 |
|------|------|
| 游客 | 10 次/天 |
| 普通登录 | 50 次/天 |
| 站长 `173236231@qq.com` | 学号二次验证后 **无限** |

服务器 `/opt/acg-api/.env` 需配置 `AUTH_OWNER_PASSWORD`、`AUTH_OWNER_SECURITY_ANSWER`（或通过 GitHub Actions Secrets 自动同步）。

## 部署检查

1. `curl https://taozhiyy.top/api/chat` → JSON 含 `remaining`
2. 主站、`/blog/`、`/build/`、`/bili` 等页面右下角可见 **✦ 问问博客**
3. 发消息能收到回复且次数递减；可问「我现在在看什么页面」验证上下文

## 安全

- 不要在前端、仓库、聊天中暴露 `DEEPSEEK_API_KEY`
- 不要使用千问 / Cursor API / 本地大模型
