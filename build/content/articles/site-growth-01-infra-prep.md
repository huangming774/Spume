---
date: 2026-06-03
slug: site-growth-01-infra-prep
title_zh: 建站成长记录 01｜前提准备：Cloudflare、UCloud 与 NameSilo 的基础搭建
title_en: Site Growth Log 01 | Foundation Setup with Cloudflare, UCloud, and NameSilo
excerpt_zh: 这一篇先不写页面，也不急着写代码，而是把我建站前真正需要准备好的三块基础资源理清楚：域名、DNS/CDN 与服务器。
excerpt_en: Before touching pages or code, this post walks through the three pieces I truly needed first: a domain, DNS/CDN, and a server.
words: 3600
reads: 168
minutes: 14
---

# 建站成长记录 01：先把地基搭好

这一组文章是我给自己的成长博客做的系列复盘。  
如果把一个个人网站比作一栋房子，那么页面设计、博客内容、接口服务都属于后面一层层往上搭的部分；而真正决定这栋房子能不能住、能不能长期稳定运行的，是最开始的基础设施。

对我来说，这个网站真正开始之前，手里已经先有了三样东西：

- `NameSilo` 域名
- `Cloudflare` DNS 与代理
- `UCloud` 云服务器

它们并不是“可选项”，而是我后面能不能把网站持续做下去的前提。

## 为什么先写这一篇

很多建站记录会直接从“我用什么框架开始写页面”讲起，但我后来越来越觉得，真正容易把人卡住的并不是 React、Hexo 或者 Go 这些代码层面的事情，而是下面这些更基础的问题：

- 域名买了之后要放在哪里管理？
- DNS 到底是谁来解析？
- HTTPS 证书谁来兜底？
- 静态页面和后端接口最终跑在什么地方？
- 后面如果我要做自动部署，服务器这一层是不是一开始就得选好？

这些问题如果前面没有理清楚，后面页面做得越多，迁移成本反而越高。

所以这篇文章的目标很明确：  
**先把网站为什么要用 `NameSilo + Cloudflare + UCloud` 这一套组合讲清楚，再把它们之间的分工讲清楚。**

## 我的基础组合

先放一张最核心的分工表。

| 组件 | 我用它做什么 | 这一层解决的问题 |
| --- | --- | --- |
| `NameSilo` | 购买和持有域名 | 让我拥有 `taozhiyy.top` 这个网站地址 |
| `Cloudflare` | 做 DNS、代理、HTTPS、加速 | 让域名能正确解析，并把访问流量更稳定地接到服务器 |
| `UCloud` | 放网站文件、跑 Nginx、跑后端服务 | 让主站、博客、成长博客和接口真正有地方运行 |
| `GitHub` | 托管仓库和 GitHub Actions | 让代码变更可以自动部署到服务器 |

如果用一句更直白的话来概括，就是：

- `NameSilo` 负责“这个名字归谁”
- `Cloudflare` 负责“别人怎么找到我”
- `UCloud` 负责“找到我以后，内容从哪里来”
- `GitHub Actions` 负责“我更新代码以后，怎么自动把新版本发到服务器”

## 为什么是 NameSilo

域名注册商有很多，最后我手里这个域名是放在 `NameSilo`。

对个人站来说，域名注册商最重要的并不是“它是不是最花哨”，而是：

1. 价格是否稳定
2. 域名管理是否清晰
3. 修改 NS 和基础解析时是否方便
4. 续费逻辑是否靠谱

我把 `NameSilo` 放在这一套链路里，主要承担的是**域名所有权入口**。  
它并不直接负责我网站的页面渲染，也不负责服务器部署，更不是我网站的应用运行环境。它最核心的作用就是：

- 我拥有这个域名
- 我可以决定这个域名交给谁做 DNS 管理
- 我可以随时把域名指向新的服务

换句话说，`NameSilo` 是**网站身份的起点**，但不是后续访问链路的核心处理层。

## 为什么还要接 Cloudflare

如果只有域名和服务器，其实理论上也能直接建站。  
但实际做下来，我更愿意把 `Cloudflare` 放在中间，原因主要有三类。

### 1. DNS 管理更集中

域名注册商和 DNS 管理不一定要是同一家。  
我更喜欢把域名托管在注册商、把解析交给更专业的 DNS 平台，这样逻辑会更清晰：

- 域名归属在 `NameSilo`
- 解析和代理在 `Cloudflare`

这样我后续改记录、接子域名、切代理、看流量，都会方便很多。

### 2. HTTPS 和代理层更省心

对个人网站来说，最怕的不是“功能不够多”，而是“每次加一个服务就多一层维护成本”。

有了 `Cloudflare` 之后，我可以把很多边缘层问题交给它：

- 站点访问走 HTTPS
- 域名请求先过代理层
- 静态内容访问体验更稳定
- 某些安全和缓存策略也可以集中处理

这意味着我的服务器不需要独自承担所有公网入口压力。

### 3. 方便后续网站逐步扩展

我这个仓库并不是一个“只有一个首页”的项目。  
它后面逐步长成了一个多模块站点：

| 路径 | 作用 |
| --- | --- |
| `/` | 主站首页与若干交互页面 |
| `/blog/` | Hexo + Butterfly 博客 |
| `/build/` | 我自己写的成长博客 |
| `/api/` | Go 后端接口 |

这种结构下，`Cloudflare` 作为入口层会让我更容易统一域名、HTTPS 和访问路径，而不是给每一块都单独找一个入口。

## 为什么服务器选 UCloud

前面域名和 DNS 都解决的是“入口问题”，但网站本身总得有地方放。  
这一层我选择的是 `UCloud` 云服务器。

原因并不复杂，因为我这个站不是单纯的静态博客，而是已经逐步发展成了“静态站点 + 动态接口 + 自动部署”的结构。

服务器这一层至少要能支撑下面几类东西：

- 静态站点文件托管
- Nginx 统一入口
- Go 后端常驻进程
- 后续自动部署脚本执行
- 若干本地数据与缓存目录

如果只是一个纯静态博客，其实 GitHub Pages、Vercel 这一类平台都很方便。  
但我这个项目后面明确需要：

- 自己的 `/api/` 服务
- 留言板数据
- AI 助手接口
- 登录与会话
- 定时同步任务

一旦有了这些需求，**拥有一台自己可控的服务器** 就会变得非常重要。

## 这一套实际是怎么串起来的

把上面这些拼在一起，我现在这套网站的访问链路就比较清楚了：

```text
用户访问 taozhiyy.top
-> 域名归属由 NameSilo 持有
-> DNS / 代理交给 Cloudflare
-> Cloudflare 回源到 UCloud 服务器
-> UCloud 上的 Nginx 分发不同路径
-> 静态页面或 /api/ 接口返回内容
```

如果再把代码部署链路补进去，就是：

```text
我在本地写代码
-> 推送到 GitHub
-> GitHub Actions 自动构建
-> 通过 SSH / rsync / scp 上传到 UCloud
-> Nginx 与后端服务完成切换
```

也就是说，我的站点不是“本地改完手传文件”的模式，而是已经走向了比较标准的工程化流程。

## 这一层准备工作实际支撑了什么

截至目前，这套基础设施已经不只是支撑一个首页，而是支撑整个站点体系。

| 模块 | 当前作用 | 依赖的基础设施 |
| --- | --- | --- |
| 主站 `main` | 展示首页、Bili 页面、留言板入口、AI 流量页 | 域名、Nginx、静态部署 |
| 博客 `blog` | Hexo + Butterfly 的文章站 | 域名、静态部署 |
| 成长博客 `build` | 记录建站过程与文章归档 | 域名、静态部署 |
| 后端 `acg-api` | 提供 `/api/` 接口 | 服务器、systemd、Nginx 反代 |
| AI 助手 | 全站右下角聊天面板 | `/api/chat`、部署环境变量 |

这也是为什么我会把“前提准备”单独写成第一篇。  
没有这一步，后面每一个功能都得临时找落点；而有了这一步，后面每长出一个模块，都能自然挂到现有结构上。

## 域名、路径和网站结构怎么设计

我现在比较喜欢的一种做法，是**一个主域名下挂多个路径模块**，而不是每做一块就单独开一个域名或者单独开一个站点。

当前这套设计大致是这样的：

| 入口 | 内容 |
| --- | --- |
| `https://taozhiyy.top/` | 主站 |
| `https://taozhiyy.top/blog/` | Hexo 博客 |
| `https://taozhiyy.top/build/` | 成长博客 |
| `https://taozhiyy.top/api/` | 后端接口 |

这种做法的好处有三点。

### 1. 统一品牌入口

所有内容都围绕一个域名展开，访问体验更完整。  
无论用户先看到首页、博客还是成长记录，都会知道它们属于同一个站点。

### 2. 运维集中

我只需要围绕同一个域名做 HTTPS、DNS、代理和服务器配置，不需要把精力分散到多个站点上。

### 3. 便于逐步扩展

我一开始可能只想做博客，但后面很容易自然延伸出：

- 个人首页
- 内容归档页
- 留言板
- 动态接口
- AI 工具

如果一开始就把结构做成“主域名 + 路径模块”，后面扩展会轻松很多。

## GitHub 在这一层为什么也重要

虽然这篇主要讲 `NameSilo + Cloudflare + UCloud`，但我还是想把 `GitHub` 单独拿出来提一句，因为它在这套站点里其实承担了一个非常关键的角色：  
**让基础设施和代码更新真正形成闭环。**

如果没有 GitHub Actions，我后续每次更新都得：

1. 本地手动构建
2. 手动登录服务器
3. 手动上传文件
4. 手动重启服务
5. 手动检查页面是否正常

这种方式在项目刚开始时可以忍，但一旦站点开始有多个模块，就会越来越痛苦。

而现在这套链路里，GitHub 负责的是：

- 记录代码历史
- 触发自动部署
- 把构建产物发往 UCloud
- 让站点更新更接近“提交即部署”

所以从工程角度看，我的基础准备并不是只有三样东西，而是四层联动：

1. `NameSilo`：域名归属
2. `Cloudflare`：DNS 与公网入口
3. `UCloud`：运行环境
4. `GitHub Actions`：更新链路

## 前期准备阶段我最看重的不是“最强”，而是“能长大”

回头看，我前期做这些选择时，并不是在追求一个“技术上最炫”的方案。  
我更在意的是，这套东西能不能支持我后面继续加内容、加页面、加服务，而不用频繁推倒重来。

所以我会把这套组合总结成一句话：

> 对个人长期项目来说，前期最重要的不是把所有技术都一次选到最满，而是先搭出一套能持续扩展的地基。

这套地基至少要满足：

- 能稳定访问
- 能统一入口
- 能方便部署
- 能支持动态服务
- 能容纳后面新增的页面和功能

就目前来看，`NameSilo + Cloudflare + UCloud + GitHub Actions` 基本满足了我的这些需求。

## 这一篇的阶段性结论

如果现在让我重新从零开始，我依然会先把下面几件事准备好，再进入真正的前端和后端开发：

1. 先拿到自己的域名
2. 先决定 DNS 和代理层放在哪里
3. 先准备一台后续能长期托管站点的服务器
4. 先想好部署是不是要自动化
5. 先决定站点后面是“只做静态内容”，还是要预留动态接口能力

因为这些决定，直接影响后面所有页面和功能的落点。

## 下一篇写什么

在基础设施确定之后，网站真正“看得见”的部分才开始出现。  
下一篇我会进入前端层，具体写：

- 主站首页是怎么做出来的
- 为什么首页前端会复刻 GitHub 上的一个开源项目
- 我是怎么把它改成自己的风格
- `Hexo + Butterfly` 为什么会成为第一套博客
- 为什么后来我又单独做了一个 `build` 成长博客

系列下一篇：

- [建站成长记录 02｜前端实现：主站、Hexo 博客与成长博客的页面设计](https://taozhiyy.top/build/post/site-growth-02-frontend-design)

---en---

# Site Growth Log 01: Build the Foundation First

This series is my structured write-up for the growth blog of my own website.  
If a personal site is like a house, then the page design, blog posts, and backend services are all the parts you build upward later. What really decides whether the house can stand, stay online, and keep evolving is the infrastructure underneath it.

Before this site truly started, I already had three key resources in place:

- a `NameSilo` domain
- `Cloudflare` for DNS and proxying
- a `UCloud` server

These were not optional extras. They were the real prerequisites for everything that came later.

## Why I started the series with infrastructure

Many site-building logs begin with the frontend stack, a framework choice, or the first page design. But the more I worked on this project, the more I felt that the real blockers usually happen earlier:

- Where should the domain actually be managed?
- Who should handle DNS?
- Who is responsible for HTTPS at the edge?
- Where should static files and backend services actually run?
- If I want automated deployment later, should the server be chosen from the start?

If these things are not clarified early, the cost of changing direction grows quickly once the site gets bigger.

So the goal of this post is simple:  
**explain why I chose the `NameSilo + Cloudflare + UCloud` combination, and explain what each layer is responsible for.**

## My base stack

Here is the most important responsibility table.

| Component | What I use it for | What problem it solves |
| --- | --- | --- |
| `NameSilo` | Buying and owning the domain | Gives me control over `taozhiyy.top` |
| `Cloudflare` | DNS, proxying, HTTPS, acceleration | Makes the domain resolve correctly and route traffic more reliably |
| `UCloud` | Hosting files, running Nginx and backend services | Gives the site a real runtime environment |
| `GitHub` | Code hosting and GitHub Actions | Makes automatic deployment possible |

In plain words:

- `NameSilo` handles ownership of the name
- `Cloudflare` handles how people reach the site
- `UCloud` handles where the actual content comes from
- `GitHub Actions` handles how new code gets deployed after I push changes

## Why NameSilo

There are many domain registrars, and in my case the domain is held at `NameSilo`.

For a personal project, what matters most in a registrar is not flashy features, but:

1. stable pricing
2. clear domain management
3. easy NS and domain-level control
4. reliable renewal behavior

Within my overall architecture, `NameSilo` is mainly the **point of ownership**.  
It does not run my pages, it does not deploy my application, and it is not my hosting environment. Its main role is:

- I own the domain
- I can decide who manages DNS
- I can redirect the domain strategy later if needed

So `NameSilo` is the starting point of the site identity, not the main runtime layer.

## Why I still put Cloudflare in front

In theory, a domain and a server are enough to get a site online.  
In practice, I prefer to place `Cloudflare` between the public internet and my server for three main reasons.

### 1. Centralized DNS control

The registrar and DNS provider do not need to be the same service.  
I prefer to keep the domain at the registrar and let a stronger DNS platform handle records and routing:

- domain ownership stays in `NameSilo`
- DNS and proxying live in `Cloudflare`

That makes record management, caching rules, proxy settings, and traffic entry much easier to reason about.

### 2. Easier HTTPS and edge handling

For a personal site, the biggest risk is not “missing a feature.”  
It is accumulating too many maintenance tasks every time the site grows.

With `Cloudflare`, I can offload a lot of edge-level concerns:

- HTTPS access
- a proxy layer in front of the server
- more stable static asset delivery
- centralized handling for some safety and caching rules

That means the server does not have to handle the entire public edge alone.

### 3. Better support for future expansion

This repository is not just a single landing page anymore.  
It has grown into a multi-part site:

| Path | Purpose |
| --- | --- |
| `/` | Main site and interactive pages |
| `/blog/` | Hexo + Butterfly blog |
| `/build/` | My custom growth blog |
| `/api/` | Go backend endpoints |

Once a site has this structure, `Cloudflare` becomes even more useful as a unified access layer.

## Why I chose UCloud for the server

The domain and DNS layers solve the entry problem.  
But the site still needs a real place to live. In my case, that place is a `UCloud` server.

This choice became necessary because the project is no longer just a static blog. It is now a combination of static sites, backend APIs, and automated deployment.

The server layer needs to support at least the following:

- hosting static build output
- running Nginx as a unified entry point
- running a persistent Go backend service
- supporting deployment scripts
- holding local data and cache directories

If the project were only a static blog, platforms like GitHub Pages or Vercel would be enough.  
But my site clearly grew beyond that. I now need:

- my own `/api/` service
- guestbook data
- AI assistant endpoints
- authentication and sessions
- scheduled synchronization tasks

Once those needs appear, having a controllable server becomes extremely valuable.

## How the whole thing connects

If I put all the layers together, the live request path looks roughly like this:

```text
User visits taozhiyy.top
-> domain ownership starts at NameSilo
-> DNS and proxying are handled by Cloudflare
-> Cloudflare forwards traffic to the UCloud server
-> Nginx routes requests by path
-> static pages or /api/ endpoints return the response
```

And if I include deployment, the flow becomes:

```text
I write code locally
-> push to GitHub
-> GitHub Actions builds the project
-> artifacts are uploaded to UCloud via SSH / rsync / scp
-> Nginx and backend services switch to the new version
```

So this is no longer a manual “edit locally and upload files by hand” site. It has already moved into a more structured engineering workflow.

## What this infrastructure already supports

At this point, the infrastructure is supporting a whole site system rather than a single page.

| Module | Current role | Infrastructure it depends on |
| --- | --- | --- |
| Main site `main` | Landing page, Bili section, guestbook entry, AI stats page | domain, Nginx, static deployment |
| Blog `blog` | Hexo + Butterfly article site | domain, static deployment |
| Growth blog `build` | Build logs and article archive | domain, static deployment |
| Backend `acg-api` | Provides `/api/` endpoints | server, systemd, Nginx reverse proxy |
| AI assistant | Global floating chat panel | `/api/chat`, deployment environment |

That is exactly why I wanted this topic to be the first post.  
Without this base layer, every new feature would need its own temporary home. With it, each new module can grow naturally into the existing structure.

## How I think about domain paths and site layout

I strongly prefer **one main domain with multiple path-based modules** instead of opening a new domain or a separate isolated site for every new idea.

My current layout is roughly:

| Entry | Content |
| --- | --- |
| `https://taozhiyy.top/` | Main site |
| `https://taozhiyy.top/blog/` | Hexo blog |
| `https://taozhiyy.top/build/` | Growth blog |
| `https://taozhiyy.top/api/` | Backend API |

This approach has three major benefits.

### 1. One consistent identity

Everything revolves around the same domain.  
Whether someone first lands on the homepage, the blog, or the build log, it still feels like the same project.

### 2. Centralized operations

I only need to think about one domain when handling HTTPS, DNS, proxying, and server behavior.

### 3. Easier long-term expansion

I may start with a simple blog, but it is natural to eventually grow into:

- a personal homepage
- content archives
- a guestbook
- dynamic APIs
- AI tools

If the structure is designed as “one domain, multiple modules” from the beginning, this evolution becomes much easier.

## Why GitHub also matters at this stage

Even though this post focuses on `NameSilo + Cloudflare + UCloud`, I still want to mention `GitHub`, because it closes the loop between code and infrastructure.

Without GitHub Actions, every update would require:

1. building locally by hand
2. logging into the server manually
3. uploading files manually
4. restarting services manually
5. verifying the site manually

That is survivable at the beginning, but it becomes painful once the project includes several modules.

In my current setup, GitHub is responsible for:

- tracking code history
- triggering automated deployment
- sending built artifacts to UCloud
- making updates feel much closer to “push and deploy”

So from an engineering perspective, the real foundation is not just three layers, but four:

1. `NameSilo` for ownership
2. `Cloudflare` for DNS and public entry
3. `UCloud` for runtime hosting
4. `GitHub Actions` for deployment

## I optimized for growth, not for maximum complexity

Looking back, I was not trying to build the flashiest or most advanced architecture on day one.  
What I cared about was whether the setup could keep growing with the project without forcing major rewrites.

So I would summarize this foundation like this:

> For a long-term personal project, the most important thing early on is not choosing the most extreme stack. It is building a base that can keep expanding.

That base should at least:

- stay reachable
- keep a unified entry
- support easy deployment
- allow dynamic services
- leave room for more pages and features later

So far, `NameSilo + Cloudflare + UCloud + GitHub Actions` has served that purpose well for me.

## Conclusion of this stage

If I had to start over from zero, I would still make the same early decisions before touching the actual frontend or backend:

1. get the domain first
2. decide who owns DNS and the proxy layer
3. prepare a server that can host the project long-term
4. decide whether deployment should be automated
5. decide whether the site will remain static-only or eventually need dynamic APIs

Those decisions shape every technical step that follows.

## What comes next

Once the infrastructure is in place, the first visible part of the site can finally emerge.  
In the next post, I will move into the frontend layer and explain:

- how the main homepage was built
- why I chose to recreate the homepage frontend from an open-source GitHub project
- how I adapted it into my own style
- why `Hexo + Butterfly` became my first blog system
- why I later built a separate `build` growth blog

Next in the series:

- [Site Growth Log 02 | Frontend Implementation: Main Site, Hexo Blog, and Growth Blog Design](https://taozhiyy.top/build/post/site-growth-02-frontend-design)
