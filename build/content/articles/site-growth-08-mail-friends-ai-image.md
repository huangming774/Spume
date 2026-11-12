---
date: 2026-06-09
slug: site-growth-08-mail-friends-ai-image
title_zh: 建站成长记录 08｜留言邮件、友链回复、服务器状态与 AI 生图上线
title_en: Site Growth Log 08 | Mail, Friend Replies, Server Status, and AI Image Generation
excerpt_zh: 这篇记录最近几次偏后端和交互闭环的更新：留言邮件通知改成 UTF-8 安全发送，友链留言补齐昵称和联系邮箱并支持连续回复，站长后台可以查看注册用户和留言邮箱，数据中心补上服务器状态监控，AI 小助手也接入 z-image-turbo 生图并把结果保存到腾讯 COS。
excerpt_en: This post records recent backend and interaction-loop updates: UTF-8-safe mail notifications, friend-page contact collection and threaded replies, owner-only email visibility, a data-center server status panel, and z-image-turbo image generation through the AI assistant with generated images saved to Tencent COS.
words: 4800
reads: 18
minutes: 15
---

# 建站成长记录 08：留言能抵达，回复能追上，图片也能生成了

这几次更新主要围绕一个目标：让网站从“能展示内容”继续往“能完成闭环”走。

之前留言墙、友链申请、AI 小助手和站长控制台已经各自存在，但它们之间还有一些断点：

- 有人留言时，站长不一定能及时知道
- 站长回复别人后，对方也不一定知道
- 友链申请需要联系邮箱，但公开页面又不应该泄露邮箱
- 留言回复只能做一层，不适合来回交流
- AI 小助手只能聊天，不能生成图像
- 生图结果如果只停留在模型临时链接里，很快就会失效

这次把这些断点逐个补上了。

## 留言邮件通知

邮件通知最先解决的是“站长能不能及时知道有人留言”。

后端现在会在创建留言后尝试发邮件：

| 场景 | 收件人 | 说明 |
| --- | --- | --- |
| 任何留言创建 | `MAIL_NOTIFY_TO` | 通知站长有新留言 |
| 站长/用户回复某条留言 | 父留言的联系邮箱，或父留言登录用户邮箱 | 告诉对方“你收到了回复” |

邮件发送规则写在后端，而不是前端。前端只提交留言内容，后端根据留言关系、登录用户和联系方式决定是否发送邮件。

这里还修了一次很重要的编码问题：邮件正文必须按 UTF-8 发送，避免中文变成一串问号。也就是说，邮件主题、正文、昵称、留言内容都必须保持中文可读。

这件事看起来只是“发邮件”，但它其实是互动闭环的开始。留言不再只是写进数据库，而是能主动抵达站长和被回复的人。

## 友链留言的联系邮箱

普通留言墙和友链留言现在被明确区分开。

普通留言墙仍然保持轻量：

- 不强制邮箱
- 主要用于公开留言和闲聊
- 登录用户使用账号昵称，匿名用户填昵称即可

友链留言则更严格：

- 昵称必填
- 邮箱必填
- 不管登录与否，友链申请都要提供联系邮箱
- 已登录用户填写了邮箱，就以填写的为准
- 邮箱只用于站长联系和回复通知，不在公开页面展示

这个区别很关键。友链申请本质上是一次站点间联系，后续可能需要站长回复、补充信息、确认链接是否已添加。如果没有邮箱，公开回复很容易变成“你看不看得到全靠缘分”。

现在友链留言表单里会明确显示昵称和邮箱，两个字段都带星号。匿名用户和登录用户都必须填写；如果只是想随便留言，说明文案会引导匿名访客前往普通留言区。

## 连续回复

友链留言还从“只能回复一次”改成了可持续回复。

之前的交互更像一问一答：有人申请，站长回复一次，流程就结束了。但实际使用里很容易出现：

- 对方补充说明
- 站长再次确认
- 对方修改链接或头像
- 站长再回复“已添加”

所以现在留言回复支持树状加载和连续追加。前端会把回复挂在对应留言下面，后端也会校验 `parentId` 和 `channel`，保证回复不会串到普通留言墙或别的友链线程里。

这让友链留言更像一个小型沟通线程，而不是一次性表单。

## 站长后台可见邮箱

联系邮箱不公开展示，但站长后台需要能看见。

这次后端补了 owner-only 的邮箱查看能力：

| 数据 | 站长可见内容 |
| --- | --- |
| 注册用户 | 用户邮箱、昵称、注册时间 |
| 留言联系人 | 留言昵称、留言位置、联系邮箱、账号邮箱回退、留言时间 |
| 站长收件箱 | 未读留言里附带可用联系邮箱 |

公开页面不会返回 `contactEmail` 字段。只有站长登录并通过权限校验后，才能从后台接口看到这些邮箱。

这条边界要非常清楚：邮箱是联系信息，不是公开内容。

## 数据中心里的服务器状态

这次也把之前“后续计划”里的服务器状态监控落到了真实页面里。

主站顶部导航现在有一个“数据中心”入口，对应 `/ai-traffic` 页面。这个页面原本主要看 AI 调用流量，现在也加入了 `ServerInfoPanel`，通过后端 `GET /api/server/info` 每 10 秒刷新一次服务器状态。

这组指标刻意只返回非敏感信息：

| 指标 | 来源/含义 | 说明 |
| --- | --- | --- |
| `status` | 服务状态 | 当前返回 `online` |
| `vendor` / `region` | 环境变量 `SERVER_VENDOR`、`SERVER_REGION` | 用于显示云厂商与区域，例如 UCloud 香港 |
| `cpuPercent` | Linux `/proc/stat` 两次采样对比 | 修过一次采样缓存逻辑，避免短时间全 idle 误显示 0 |
| `memory` | Linux `/proc/meminfo` | 优先读系统内存；本地开发环境降级为 Go 进程内存 |
| `uptime` / `uptimeSecs` | API 进程启动时间 | 展示服务持续运行时长 |
| `os` / `arch` / `goVersion` | 运行时信息 | 只显示友好的系统名和 Go 版本 |
| `cpuCores` / `goroutines` | Go runtime | 展示核心数与当前 goroutine 数 |
| `serverTime` | UTC 时间 | 用于确认遥测刷新时间 |

这里最重要的不是“炫一个仪表盘”，而是边界：接口故意不返回真实 IP、主机名、域名、磁盘路径、进程列表、环境变量等敏感信息。前端展示的是安全摘要，不是服务器控制权。

前端视觉上用了四个圆形 HUD：CPU、内存、运行时长和运行环境。在线状态、云厂商区域和 serverTime 放在底部小状态条里。这样站长不用 SSH 到服务器，也能在页面上快速判断“服务还活着、资源有没有异常、遥测有没有更新”。

## AI 生图接入

AI 小助手现在不只支持聊天，也支持“生图”模式。

实现路径是：

1. 前端在小助手里切换 `聊天 / 生图`
2. 用户输入画面描述
3. 后端检查登录态和额度
4. 调用阿里云百炼 DashScope 的 `z-image-turbo`
5. 固定关闭提示词改写，保持 `promptExtend: false`
6. 后端下载模型生成的临时图片
7. 使用服务器端腾讯 COS 凭据上传图片
8. 返回最终 COS 链接给前端
9. 前端用浅色透明镭射卡片在屏幕中央展示图片

我选择把图片保存到 COS，而不是直接用模型返回的临时地址，是因为临时地址不可控。真正进入网站体验的结果应该是稳定的公开链接，方便复制、保存和后续整理。

## 额度和安全边界

生图不是完全公开接口。

当前规则是：

| 用户类型 | 生图能力 |
| --- | --- |
| 未登录访客 | 不允许生图 |
| 普通登录用户 | 每天 3 张 |
| 站长账号 | 不限额度 |

这个限制有两个目的：

1. 避免费用被刷
2. 让每张图都能关联到登录用户或明确身份

另外，阿里云百炼 Key、腾讯 COS Secret、SMTP 授权码都只放在服务器环境或 GitHub Actions Secrets 里。仓库里只保留环境变量名和配置说明，不提交真实密钥。

## 前端展示和操作

生图完成后，图片不会只出现在聊天气泡里，而是打开一个屏幕中央的结果卡片。

这个卡片沿用了 Source Lottery 壁纸弹窗的方向：

- 透明浅色背景
- 镭射质感
- 图片居中展示
- 移动端保持可用尺寸
- 两个动作：保存本地、复制链接

按钮规则也改清楚了：

| 按钮 | 行为 |
| --- | --- |
| 保存本地 | 优先下载 COS 图片；如果浏览器或 COS CORS 限制下载，则打开原图链接兜底 |
| 复制链接 | 复制后端返回的最终 COS 链接 |

这里的“复制链接”不是复制模型临时地址，而是复制上传到腾讯 COS 后的稳定 URL。

## README 和 build 记录为什么也要更新

这几次改动涉及的不只是页面样式，而是后端能力边界：

- SMTP 邮件通知
- 友链联系邮箱
- 连续回复
- 站长后台邮箱查看
- 数据中心服务器状态监控
- 百炼生图
- COS 保存生成图
- GitHub Actions Secrets 部署配置

如果 README 不更新，后续自己回来看也会误判当前系统状态。

尤其“邮箱通知”、“服务器状态监控”和“低成本生图”已经从计划变成了已上线能力，所以后续计划里不应该再保留这些已完成项。

## 接下来的计划

后续计划现在更聚焦：

| 计划 | 想解决的问题 |
| --- | --- |
| 服务器历史趋势与告警 | 在当前实时状态面板基础上继续记录 CPU、内存、错误率和同步耗时趋势 |
| 打造生成图展示页面 | 把已经生成并保存到 COS 的图片整理成一个可浏览、可回看、可管理的展示页 |
| 设计艺术字主页 | 继续探索更有辨识度的首页标题、艺术字和主视觉表达 |

其中“打造生成图展示页面”是 AI 生图上线之后自然出现的下一步。

现在图片已经能生成、能保存、能复制链接，但它们还只是一次性结果。下一步应该让这些图成为站内内容资产：能查看历史、挑选好图、公开展示，甚至未来可以加精选、标签和删除管理。

这比继续堆更多模型更重要。先让已经生成的东西被好好保存和展示，网站才会真的长出自己的素材库。

---en---

# Site Growth Log 08: Messages Can Arrive, Replies Can Follow, and Images Can Be Generated

These updates focus on closing interaction loops.

The site already had a guestbook, friend applications, an AI assistant, and an owner console, but several pieces were still disconnected:

- the owner might not notice a new message in time
- a visitor might not know when the owner replies
- friend applications need contact email, but public pages should not expose it
- replies should allow a real back-and-forth thread
- the AI assistant could chat but could not generate images
- generated-image URLs should not depend on temporary provider links

This round connects those pieces.

## Mail notifications

The first problem was simple: when someone leaves a message, the owner should know.

The backend now attempts mail notification after message creation:

| Event | Recipient | Notes |
| --- | --- | --- |
| Any new message | `MAIL_NOTIFY_TO` | Notifies the owner |
| A reply to an existing message | the parent message contact email, or the parent author's account email | Notifies the person being replied to |

The rule lives on the backend. The frontend submits message content; the backend decides whether to send mail based on message channel, parent message, login state, and stored contact information.

A key fix here was UTF-8 correctness. Chinese subject lines, nicknames, and message bodies must remain readable instead of becoming question marks.

## Friend-page contact email

The normal guestbook and the friend-page guestbook now have different rules.

The normal guestbook stays lightweight:

- no required email
- anonymous visitors can leave a nickname
- logged-in users use their account display name

Friend-page messages are stricter:

- nickname is required
- email is required
- this applies to both anonymous and logged-in users
- if a logged-in user enters a contact email, that email wins
- the email is collected for owner contact and reply notification only, never shown publicly

Friend applications are closer to site-to-site contact than casual comments, so a reply channel matters.

## Threaded replies

Friend-page messages also moved from one-off replies to continuous threaded replies.

A real friend-link conversation may need follow-up:

- the visitor adds missing information
- the owner confirms the link
- the visitor changes avatar or URL
- the owner replies that the link has been added

The frontend now appends replies under the relevant thread, and the backend validates `parentId` and `channel` so replies do not leak across the normal guestbook and friend-page threads.

## Owner-only email visibility

Contact email is private, but the owner needs to see it.

The backend now exposes owner-only email views:

| Data | Owner-visible fields |
| --- | --- |
| Registered users | email, display name, registration time |
| Guestbook contacts | nickname, channel, contact email, account-email fallback, message time |
| Owner inbox | unread messages with usable contact email |

Public APIs do not return `contactEmail`. Only an owner session can see these fields.

## Server status in the data center

This round also turned the earlier server-status plan into a real page.

The main navigation now has a Data Center entry at `/ai-traffic`. That page already visualizes AI traffic, and now it also renders `ServerInfoPanel`, which refreshes `GET /api/server/info` every 10 seconds.

The endpoint deliberately returns only non-sensitive telemetry:

| Metric | Source / meaning | Notes |
| --- | --- | --- |
| `status` | service state | currently returns `online` |
| `vendor` / `region` | `SERVER_VENDOR` and `SERVER_REGION` env vars | used for cloud vendor and region display, such as UCloud Hong Kong |
| `cpuPercent` | two-sample Linux `/proc/stat` comparison | the sampling cache was fixed to avoid short all-idle windows showing a misleading 0 |
| `memory` | Linux `/proc/meminfo` | system memory first; local development falls back to Go process memory |
| `uptime` / `uptimeSecs` | API process start time | shows how long the service has been running |
| `os` / `arch` / `goVersion` | runtime metadata | friendly OS name and Go version only |
| `cpuCores` / `goroutines` | Go runtime | core count and current goroutine count |
| `serverTime` | UTC time | confirms telemetry freshness |

The key point is the boundary. The endpoint intentionally does not return real IPs, hostnames, domains, disk paths, process lists, environment variables, or other sensitive server details. It is a safe status summary, not server control.

Visually, the frontend presents four circular HUD cards: CPU, memory, uptime, and runtime. Online state, cloud region, and server time sit in a small footer strip. This lets the owner quickly check whether the service is alive, whether resources look abnormal, and whether telemetry is still refreshing, without SSHing into the server.

## AI image generation

The floating AI assistant now has an image-generation mode.

The flow is:

1. switch the assistant from Chat to Image
2. enter an image prompt
3. the backend checks login state and quota
4. the backend calls Alibaba Cloud DashScope `z-image-turbo`
5. prompt rewriting is disabled with `promptExtend: false`
6. the backend downloads the temporary provider image
7. the backend uploads it to Tencent COS with server-side credentials
8. the frontend receives the final COS URL
9. a translucent holographic result card opens in the center of the screen

Saving the result to COS is important because provider URLs are temporary. A generated image should become a stable site asset, not a disappearing response.

## Quota and security boundaries

Image generation is not a public anonymous endpoint.

| User type | Image generation |
| --- | --- |
| Anonymous visitor | not allowed |
| Logged-in user | 3 images per day |
| Owner | unlimited |

DashScope keys, COS secrets, and SMTP authorization codes stay in server environment variables or GitHub Actions Secrets. The repository only documents variable names and behavior.

## Result card actions

Generated images open in a centered translucent card, visually aligned with the Source Lottery wallpaper modal.

The action buttons are:

| Button | Behavior |
| --- | --- |
| Save locally | tries to download the COS image; if CORS or browser rules block it, opens the image URL as fallback |
| Copy link | copies the final COS URL returned by the backend |

The copied link is the stable COS URL, not the temporary provider URL.

## Documentation update

These changes affect backend responsibilities and deployment configuration:

- SMTP notifications
- friend-page contact email
- threaded replies
- owner-only email views
- data-center server status monitoring
- DashScope image generation
- COS storage for generated images
- GitHub Actions Secrets

That is why the README and build log were updated as part of the work.

Email notifications, server status monitoring, and low-cost image generation are no longer future plans. They are implemented capabilities now.

## Next steps

The next plan is more focused:

| Plan | Goal |
| --- | --- |
| Server history and alerts | keep trend records for CPU, memory, error rate, and sync duration on top of the current live status panel |
| Generated-image showcase page | turn generated COS images into a browsable, reusable, and manageable site gallery |
| Artistic lettering homepage | keep exploring a more recognizable title, lettering system, and main visual identity |

The generated-image showcase is the natural next step after image generation.

Right now, the assistant can generate an image, save it, and return a stable link. The next layer is to make those images visible again later: browse history, pick good results, publish selected images, and eventually add tags or owner-side cleanup.

That matters more than adding more models immediately. A site grows stronger when the things it creates become organized assets.
