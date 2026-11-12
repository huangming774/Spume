---
date: 2026-06-03
slug: site-growth-03-backend-services
title_zh: 建站成长记录 03｜后端实现：接口、登录、留言板与 AI 助手
title_en: Site Growth Log 03 | Backend Implementation for APIs, Login, Guestbook, and the AI Assistant
excerpt_zh: 这一篇整理当前网站后端已经落地的能力：Go API、留言板、登录注册、站长二次验证、AI 助手额度控制，以及自动部署到 UCloud 的整体链路。
excerpt_en: This post summarizes the backend features already in place: the Go API, guestbook, login and registration, owner verification, AI quota control, and automated deployment to UCloud.
words: 4700
reads: 141
minutes: 19
---

# 建站成长记录 03：网站真正“活起来”的那部分

前两篇我分别写了：

- 基础设施怎么搭
- 前端页面怎么长出来

如果说第一篇解决的是“网站能不能稳定上线”，第二篇解决的是“网站长什么样”，那么这一篇要解决的问题就是：

> 这个网站为什么不只是一个静态页面集合，而是开始具备真正的交互能力？

答案就在后端。

对我现在这套站来说，后端并不是一个非常庞大的业务系统，但它已经承担了很多关键角色：

- 给前端页面提供接口
- 支撑留言板
- 提供登录、注册和会话
- 处理 AI 助手请求与额度控制
- 定时同步部分外部内容
- 与部署流程联动，成为一个真正长期运行的服务

这篇文章我会把“当前已经做出来的后端”完整整理一下。

## 当前后端的核心形态

从仓库结构看，我现在的后端核心是 `acg-api` 这个 Go 服务。  
它不是挂在某个第三方函数平台上，而是作为一个长期运行的服务部署在服务器上。

这意味着它具备几种很重要的能力：

- 可以常驻运行
- 可以保存本地数据
- 可以管理会话
- 可以通过 systemd 管理生命周期
- 可以通过 Nginx 统一暴露在 `/api/` 路径下

这对整个网站来说是一个分水岭。  
一旦后端常驻服务稳定存在，网站就不再只是“静态页面 + 外链跳转”，而是开始拥有真正的应用能力。

## 我为什么没有把后端拆得很重

回头看，我现在这套后端其实走的是一个很实用的路线：  
**优先做一个够用、清晰、能部署、能扩展的服务，而不是先把架构复杂化。**

它目前的特点大概可以概括为：

- 单个 Go 服务承载主要接口
- SQLite 负责本地数据落盘
- Nginx 负责统一入口与反代
- systemd 负责进程保活

对个人项目来说，这种方案非常合适。

原因也很简单：

1. 足够轻
2. 足够稳
3. 部署链路简单
4. 扩展一个接口的成本不高

如果一开始就引入更重的微服务、容器编排或者复杂中间层，反而会让维护负担远大于收益。

## 当前后端已经提供了哪些能力

先给出一张总览表，把目前已经落地的接口能力按功能分组列出来。

| 分类 | 当前能力 | 前台对应功能 |
| --- | --- | --- |
| 健康检查 | 服务状态接口 | 部署后校验 |
| 留言板 | 留言列表、发布、隐藏、删除 | `/guestbook` |
| 番剧/动态 | 番剧列表、雷达流、同步触发 | `/bili` 等页面 |
| 图片缓存 | 本地图片缓存读取 | ACG 内容展示 |
| 登录系统 | 注册、登录、登出、当前登录态 | AI 助手与留言板 |
| 站长验证 | 二次安全问题验证 | 站长无限额度 |
| AI 接口 | 查询额度、发起聊天、统计数据 | 全站 AI 助手、`/ai-traffic` |

如果只从“网站已经有多少动态能力”这个角度看，其实现在这套后端已经不算简单了。

## `/api/` 这一层为什么重要

对用户来说，很多功能看起来只是页面按钮，但对我来说，真正让这些按钮有意义的是 `/api/` 这一层。

因为一旦我把动态能力统一收口到 `/api/`，前端与服务端的边界就清晰了：

- 前端负责展示和交互
- 后端负责状态、数据和权限

这样后面无论我加的是：

- 新页面
- 新统计
- 新权限逻辑
- 新同步任务

都可以优先考虑“接口应该长成什么样”，而不是把所有逻辑硬塞到前端。

## 留言板是怎么支撑起来的

留言板是这套网站里非常典型的一类“轻交互功能”。

从用户视角看，它只是一个可以发消息、看到别人留言的页面。  
但从实现角度看，它其实已经包含了不少后端关心的问题：

- 数据如何存储
- 游客和登录用户怎么区分
- 管理员怎样隐藏或删除内容
- 怎样防止刷屏
- IP 相关信息如何处理

### 留言板当前提供的能力

目前留言板已经具备的功能包括：

| 能力 | 说明 |
| --- | --- |
| 留言分页列表 | 按页加载历史留言 |
| 游客留言 | 游客可以填写昵称和内容 |
| 登录用户留言 | 登录后可直接以账号身份留言 |
| 管理员隐藏 | 管理员可把留言设为隐藏 |
| 管理员删除 | 管理员可删除留言 |
| 地区展示 | 留言可带出地区信息 |
| 限流 | 针对频率和重复内容做限制 |

这套功能说明一个很重要的事实：  
后端不是只负责“存一条文本”，而是要把留言板当作一个最小可用系统来处理。

### 为什么留言板值得做成独立系统

留言板看起来小，但它很能锻炼一个网站最基础的动态能力。

因为它天然会牵涉这些问题：

- 读写接口
- 分页
- 用户身份
- 管理员权限
- 安全策略
- 内容展示格式

如果这部分能做扎实，后面很多更复杂的交互功能也会更容易继续长出来。

## 登录、注册和会话系统现在做到了哪里

对我来说，登录系统最初并不是为了做一个完整用户平台，而是为了给 AI 助手和部分交互功能提供更合理的身份区分。

当前已经有这些基础能力：

| 接口 | 作用 |
| --- | --- |
| `POST /api/auth/register` | 注册用户 |
| `POST /api/auth/login` | 登录 |
| `POST /api/auth/logout` | 登出 |
| `GET /api/auth/me` | 获取当前登录态 |
| `POST /api/auth/verify-security` | 站长二次验证 |

这说明后端现在已经不是“完全游客态”的结构，而是有了明确的用户身份层。

### 这套登录系统的实际价值

它至少解决了三件事：

1. 把游客和登录用户区分开
2. 让后端可以根据身份发不同额度和权限
3. 为后续更复杂的用户能力留出接口基础

对于一个个人网站来说，这已经很有用了。  
因为很多功能并不需要完整社交系统，但非常需要最基本的身份识别。

## 为什么还要有“站长二次验证”

这一点是我觉得比较有意思，也比较符合个人站特征的一块设计。

当前这套系统里，站长账号并不是普通登录后就完全等同于一般用户，而是额外有一层安全问题验证。  
这层验证通过后，站长可以拿到更高权限，比如无限 AI 额度。

从工程设计角度，这样做有几个好处。

### 1. 区分普通登录与高权限使用

不是所有登录都应该直接等于完全管理权限。  
多一层验证，能把“我只是登录了”与“我是站长本人”区分开。

### 2. 更适合个人网站场景

对一个个人项目来说，未必需要复杂的后台管理系统，但往往需要一种轻量办法来识别“真正的自己”。

### 3. 给 AI 额度控制留下合理分层

因为 AI 调用涉及额度和成本，站长身份做额外验证是很合理的。

## AI 助手现在已经做到了什么

如果说留言板代表的是“社区感”的动态能力，那么 AI 助手代表的就是“站点工具化”的动态能力。

现在这套网站的 AI 助手已经不是一个纯前端假组件，而是由后端真实支撑起来的。

它当前大致做了这些事：

| 能力 | 说明 |
| --- | --- |
| 查询今日额度 | 前端可先获取剩余额度 |
| 发起聊天 | 前端把问题发给 `/api/chat` |
| 记录页面上下文 | 会携带当前页面标题和 URL |
| 调用上游模型 | 由后端转发给 DeepSeek |
| 返回回复 | 再由后端返回给前端面板 |
| 统计请求结果 | 成功、失败、限额、限流都会统计 |

这一点很重要，因为它意味着：

- API Key 不会暴露在前端
- 额度控制在后端执行
- 登录身份也由后端统一判断

从安全和运维角度看，这比前端直连模型服务靠谱得多。

## AI 额度控制为什么要放在后端

这个问题我很想单独写清楚。

如果 AI 助手只是一个前端输入框，最简单的做法当然是直接前端请求模型接口。  
但这种方式有几个明显问题：

- API Key 暴露风险
- 无法可靠限制调用次数
- 难以区分游客与登录用户
- 统计数据不完整

所以我现在选择的方案是：

1. 前端只负责收集问题和页面上下文
2. 后端负责判断身份、频率和额度
3. 后端再去调用上游模型
4. 调用结果统一回到前端

这种结构的价值非常直接：

- 更安全
- 更可控
- 更便于统计
- 更便于后续加规则

## AI 流量统计页为什么也是后端价值的一部分

网站里现在还有一个 `/ai-traffic` 页面。  
表面看它是前端统计展示页，但本质上它依赖的是后端已经把调用数据结构化保存下来了。

这说明后端现在不仅是在“提供一个聊天接口”，而是在逐步形成一套**可观测的 AI 调用层**。

这很有价值，因为只要有统计，就能继续做：

- 每日趋势分析
- 失败率判断
- 登录用户与游客占比分析
- 后续额度策略优化

对一个持续运行的网站来说，能看见系统行为，本身就是一种能力。

## 番剧与动态同步这一块的意义

后端当前还有一类很有代表性的功能，就是 B 站相关内容同步。

从站点表现看，它主要服务于：

- 番剧列表
- 创作者雷达流
- 部分图片缓存内容

这部分说明后端除了处理“用户交互”，也在承担“外部内容整合”的职责。

这类能力的意义在于：

- 前端不必每次都直接请求外部源
- 后端可以整理、缓存和统一输出
- 页面展示更稳定

对个人站来说，这是一种非常实用的后端扩展方向。

## 数据存储为什么走 SQLite

我现在这套后端没有为了“看起来更像大系统”就强上更重的数据库方案，而是用了 SQLite。

这在个人项目里其实非常务实。

原因包括：

1. 部署简单
2. 备份思路清晰
3. 读写量在当前规模下完全足够
4. 与 Go 服务整合成本低

对于目前这些功能来说：

- 留言板
- 用户登录
- 会话
- AI 统计
- 同步数据

SQLite 已经足够支撑，而且会让整个系统更轻。

## 后端是怎么在服务器上跑起来的

当前这套后端并不是“上传一个二进制然后手动跑一下”那么简单。  
它已经接入了比较标准的服务器运行方式。

目前大致链路是：

1. GitHub Actions 构建 Linux 版 `acg-api`
2. 把二进制、service 文件和脚本传到 UCloud
3. 远端脚本安装到固定目录
4. `systemd` 接管进程
5. `Nginx` 把 `/api/` 反代到本机服务

这一整套流程意味着：

- 服务重启更规范
- 开机自启可控
- 统一从域名路径访问
- 部署可重复执行

这已经不是“实验脚本”级别，而是很接近长期维护项目的方式。

## GitHub Actions 与后端部署的关系

前面两篇都提到过 GitHub Actions，但到了后端这里，它的重要性会更明显。

因为后端更新如果不自动化，成本会更高：

- 二进制要重新构建
- 配置和脚本要重新传
- 服务要重启
- 页面和接口要回归检查

而现在这套流程已经把这些动作串起来了。  
从工程角度看，这带来的不是“方便一点”，而是**敢继续迭代**。

因为一旦部署链路稳定，后端就不再是“我不敢动的那部分”，而会变成“我可以持续往上加功能的那部分”。

## 当前后端最像一个什么阶段

如果要给现在这套后端定一个阶段，我会说它已经从“只有几个辅助接口”进入了“轻量个人应用后端”的阶段。

它的特点非常清楚：

- 有真实用户交互
- 有身份体系
- 有权限差异
- 有外部内容同步
- 有 AI 能力接入
- 有部署自动化

虽然它远远不是那种庞大的商业后台系统，但对于一个个人网站来说，它已经足够构成“网站活起来的核心部分”。

## 截至目前，这套后端已经支撑了哪些页面

为了让前后端关系更直观，我再放一张页面对应表。

| 页面或功能 | 依赖的后端能力 |
| --- | --- |
| `/guestbook` | 留言列表、发布、管理、身份判断 |
| `/bili` | 番剧与动态相关数据输出 |
| 全站 AI 助手 | 聊天接口、额度控制、登录状态 |
| `/ai-traffic` | AI 调用统计聚合 |
| 管理相关操作 | 站长身份和权限判断 |

这张表很能说明问题：  
现在的网站已经不是“页面先写好，后端随便补一下”，而是很多页面已经明确依赖后端才能成立。

## 我对这一阶段后端建设的理解

这阶段我最大的感受是：

> 对个人项目来说，后端不一定要大，但一定要让前端真正拥有可持续的状态、身份和数据来源。

这正是我现在这套后端的意义。

它不是为了炫技而复杂化，而是让网站开始具备：

- 可交互
- 可管理
- 可统计
- 可扩展

这些能力一旦出现，网站就从“展示项目”更明显地走向了“可运行项目”。

## 这一组系列的阶段性收尾

到这里，这组三篇“建站成长记录”的第一轮主线就基本完整了。

1. 第一篇讲的是基础设施：域名、Cloudflare、服务器与部署入口
2. 第二篇讲的是前端：主站、Hexo 博客和成长博客的形成
3. 第三篇讲的是后端：接口、登录、留言板、AI 助手与自动部署

如果以后继续往下写，我很可能会再拆出更多单独主题，比如：

- 留言板系统如何继续完善
- AI 助手提示词与统计策略怎么演化
- `build` 成长博客的 Markdown 生成链路如何继续升级
- 整个站点未来怎样继续整理成更统一的架构

系列上一篇：

- [建站成长记录 02｜前端实现：主站、Hexo 博客与成长博客的页面设计](https://taozhiyy.top/build/post/site-growth-02-frontend-design)

---en---

# Site Growth Log 03: The Part That Makes the Website Feel Alive

The first two posts in this series covered:

- how the infrastructure was prepared
- how the frontend layers grew into shape

If the first post solved “can the site stay online,” and the second solved “what does the site look like,” then this post answers a different question:

> Why is this site no longer just a collection of static pages, but something that now supports real interaction?

The answer is the backend.

My current backend is not a huge business system, but it already takes on several important roles:

- providing APIs for the frontend
- supporting the guestbook
- handling login, registration, and sessions
- processing AI assistant requests and quota control
- synchronizing selected external content
- participating in a real automated deployment workflow

This post is a full summary of the backend that already exists.

## The current backend shape

At the center of the backend is a Go service called `acg-api`.  
It is not running as a temporary serverless endpoint. It is deployed as a long-running service on the server.

That gives it several important qualities:

- it can stay alive continuously
- it can persist local data
- it can manage sessions
- it can be controlled by systemd
- it can be exposed under `/api/` through Nginx

For the site as a whole, that is a major transition point.  
Once a stable backend service exists, the website becomes more than static pages with links. It starts behaving like an application.

## Why I kept the backend lightweight

Looking back, the backend follows a very practical route:  
**build something clear, deployable, and extensible before making it complicated.**

Its current structure is roughly:

- one Go service for the main APIs
- SQLite for local persistence
- Nginx as the unified entry and reverse proxy
- systemd for process lifecycle

For a personal project, this is a very strong balance.

The reasons are simple:

1. it is lightweight
2. it is stable
3. the deployment path is easy to reason about
4. adding new endpoints is not too expensive

If I had started with heavier service splitting, orchestration, or unnecessary middle layers, the maintenance cost would have grown faster than the value.

## What the backend already provides

Here is a grouped overview of the features already implemented.

| Category | Current ability | Frontend feature it supports |
| --- | --- | --- |
| Health check | service status endpoint | deployment verification |
| Guestbook | list, post, hide, delete | `/guestbook` |
| Bangumi / feed | bangumi list, radar feed, sync trigger | `/bili` and related pages |
| Image cache | local cached image serving | ACG-related content |
| Authentication | register, login, logout, current session | AI assistant and guestbook |
| Owner verification | secondary security verification | unlimited owner quota |
| AI endpoints | quota query, chat request, usage stats | site-wide AI assistant and `/ai-traffic` |

So from the point of view of dynamic behavior, the backend is already doing quite a lot.

## Why the `/api/` layer matters

From the user’s point of view, many features look like simple buttons and pages.  
From my point of view, what gives those features meaning is the `/api/` layer underneath.

Once dynamic behavior is consistently placed under `/api/`, the boundary becomes much clearer:

- the frontend handles presentation and interaction
- the backend handles state, data, and permissions

That makes future work easier. Whether I add:

- a new page
- new statistics
- new permission logic
- new synchronization jobs

I can think in terms of “what should the API look like?” instead of trying to push everything into the frontend.

## How the guestbook is supported

The guestbook is a very representative lightweight interactive system in this site.

From the user side, it is simply a page where people can leave messages and read others.  
From the implementation side, it already touches a surprising number of backend concerns:

- how data is stored
- how guests and logged-in users are distinguished
- how admin actions work
- how spam and abuse are limited
- how IP-related information is handled

### Current guestbook abilities

The guestbook currently includes:

| Ability | Description |
| --- | --- |
| paginated list | load messages page by page |
| guest posting | guests can submit nickname + content |
| logged-in posting | signed-in users can post directly |
| admin hide | admins can hide a message |
| admin delete | admins can remove a message |
| region display | message cards can show region information |
| rate limiting | limits based on frequency and repeated content |

That makes one thing very clear:  
the backend is not just storing text rows. It is treating the guestbook like a real minimal interaction system.

### Why the guestbook matters

A guestbook may look small, but it exercises many of the most important backend basics:

- read/write APIs
- pagination
- identity
- admin permissions
- safety rules
- display-oriented data shaping

If this layer is solid, many future interactive features become easier to add.

## Login, registration, and sessions

For me, authentication was not originally about building a complete user platform.  
It was mainly introduced to support more meaningful identity distinctions for the AI assistant and some interactive features.

The current base capabilities include:

| Endpoint | Purpose |
| --- | --- |
| `POST /api/auth/register` | create a user account |
| `POST /api/auth/login` | sign in |
| `POST /api/auth/logout` | sign out |
| `GET /api/auth/me` | return current login state |
| `POST /api/auth/verify-security` | owner secondary verification |

This means the backend is no longer purely anonymous. It now has a distinct identity layer.

### Why this authentication layer matters

It already solves at least three useful problems:

1. it distinguishes guests from logged-in users
2. it allows different quotas and permissions by identity
3. it creates a foundation for richer user-based features later

For a personal site, that is already very meaningful.  
Many features do not need a full social platform, but they do benefit from basic identity handling.

## Why there is also owner verification

This is one of the more interesting parts of the system and one that fits a personal site very well.

In the current design, the owner account is not treated the same as an ordinary logged-in user.  
It goes through an extra security-question verification step. After that, it can unlock higher privileges such as unlimited AI usage.

From a design perspective, this has several benefits.

### 1. It separates ordinary login from high-trust usage

Not every login should automatically grant the same power.  
An extra verification step creates a distinction between “someone signed in” and “this is truly the site owner.”

### 2. It fits a personal-site model well

For a personal project, I may not need a huge admin system, but I do need a lightweight way to verify “this is really me.”

### 3. It gives AI quota control a more reasonable hierarchy

Since AI requests relate to usage cost, a stronger owner distinction makes sense.

## What the AI assistant already does

If the guestbook represents community-style interactivity, then the AI assistant represents the site becoming tool-like and service-backed.

The assistant is no longer a fake frontend widget. It is backed by a real server flow.

Right now it does roughly the following:

| Ability | Description |
| --- | --- |
| quota query | frontend can fetch today’s remaining quota |
| chat request | frontend sends questions to `/api/chat` |
| page context capture | includes current page title and URL |
| upstream model call | backend forwards the request to DeepSeek |
| reply return | backend sends the response back to the panel |
| request statistics | success, failure, quota, and rate-limit outcomes are tracked |

This matters because:

- the API key stays off the frontend
- quota control lives on the backend
- identity checks are centralized

From both security and operations perspectives, that is far more reliable than calling the model directly from the browser.

## Why AI quota control belongs on the backend

I want to make this point explicit.

If the assistant were only a frontend text box, the simplest version would be to call the model directly from the browser.  
But that creates obvious problems:

- API key exposure
- unreliable usage limits
- weak guest vs user distinction
- incomplete statistics

So the architecture I use is:

1. the frontend only gathers the question and page context
2. the backend checks identity, frequency, and quota
3. the backend calls the upstream model
4. the result returns through the backend

The value is immediate:

- safer
- more controllable
- easier to measure
- easier to evolve later

## Why the AI traffic page is also a backend achievement

The site currently includes an `/ai-traffic` page.  
On the surface, it looks like a frontend statistics view, but its existence depends on the backend already saving structured usage data.

That means the backend is not merely exposing a chat endpoint. It is becoming an **observable AI usage layer**.

That is valuable because once those numbers exist, I can keep building on them:

- daily trend analysis
- failure-rate analysis
- guest vs signed-in usage distribution
- future quota policy tuning

For a site that keeps running, being able to see system behavior is itself a feature.

## The role of bangumi and feed synchronization

Another representative backend feature is the synchronization of Bilibili-related content.

At the site level, this mainly supports:

- bangumi lists
- creator radar feeds
- some cached image content

This shows that the backend is not only handling user interaction. It is also handling **external content integration**.

That kind of capability is useful because:

- the frontend does not need to hit external sources every time
- the backend can normalize and cache data
- page rendering becomes more stable

For a personal site, that is a very practical direction for backend expansion.

## Why SQLite is enough here

I did not switch to a heavier database just to make the project look more “serious.”  
The backend currently uses SQLite, which is a very practical choice for this scale.

Reasons include:

1. easy deployment
2. clear backup thinking
3. more than enough performance for the current scale
4. low integration overhead with Go

For the current feature set:

- guestbook
- user accounts
- sessions
- AI stats
- synchronized data

SQLite is entirely sufficient and keeps the whole system lighter.

## How the backend actually runs on the server

The backend is no longer just “upload a binary and run it manually once.”  
It is already connected to a more standard runtime pattern.

The rough flow now is:

1. GitHub Actions builds a Linux binary of `acg-api`
2. the binary, service file, and scripts are uploaded to UCloud
3. a remote script installs them into stable locations
4. systemd manages the process
5. Nginx proxies `/api/` to the local service

This means:

- restarts are more standardized
- boot-time startup is manageable
- all access stays under the same domain
- deployment is repeatable

That moves the backend much closer to a long-term maintainable service.

## GitHub Actions and backend deployment

The previous posts already mentioned GitHub Actions, but it becomes even more important here.

Backend changes are more expensive to ship manually because:

- the binary must be rebuilt
- scripts and service files may need updating
- the service must restart
- the live site needs health checks afterward

The current deployment workflow already connects those pieces.  
From an engineering perspective, the benefit is not just convenience. It is confidence.

Once the deployment path is reliable, the backend stops being “the part I am afraid to touch” and becomes “the part I can keep growing.”

## What stage the backend is in now

If I had to describe the current backend stage, I would call it a **lightweight personal application backend** rather than a few helper endpoints.

Its characteristics are clear:

- real user interaction
- identity and sessions
- permission differences
- external content synchronization
- AI integration
- automated deployment

It is far from a huge commercial backend, but for a personal site it already forms the core of what makes the project alive.

## Which pages already depend on it

To make the frontend-backend relationship clearer, here is a simple mapping table.

| Page or feature | Backend ability it depends on |
| --- | --- |
| `/guestbook` | listing, posting, moderation, identity checks |
| `/bili` | bangumi and feed-related data |
| global AI assistant | chat endpoint, quota control, login state |
| `/ai-traffic` | aggregated AI usage stats |
| admin-like actions | owner identity and permission rules |

This table highlights something important:  
the site is no longer built as “frontend first, backend maybe later.” Many visible parts now depend on backend behavior to make sense.

## My main lesson from this backend stage

My biggest takeaway is this:

> For a personal project, the backend does not need to be huge, but it does need to give the frontend sustainable state, identity, and data.

That is exactly what this backend layer now provides.

It was not made complicated for show. It was built so the site could become:

- interactive
- manageable
- measurable
- extensible

Once those qualities appear, a website starts moving from “a showcase project” toward “a running project.”

## Closing this first growth trilogy

At this point, the first main arc of this “site growth log” trilogy is complete.

1. The first post covered infrastructure: domain, Cloudflare, server, and deployment entry.
2. The second covered the frontend: the main site, Hexo blog, and growth blog structure.
3. This third post covered the backend: APIs, login, guestbook, AI assistant, and automated deployment.

If I keep writing beyond this point, I will likely split future posts into more focused topics, such as:

- how the guestbook system can keep evolving
- how prompts and AI statistics strategy may develop
- how the `build` content pipeline can be improved further
- how the whole site may be reorganized into a more unified architecture

Previous in the series:

- [Site Growth Log 02 | Frontend Implementation for the Main Site, Hexo Blog, and Growth Blog](https://taozhiyy.top/build/post/site-growth-02-frontend-design)
