---
date: 2026-06-03
slug: site-growth-02-frontend-design
title_zh: 建站成长记录 02｜前端实现：主站、Hexo 博客与成长博客的页面设计
title_en: Site Growth Log 02 | Frontend Implementation for the Main Site, Hexo Blog, and Growth Blog
excerpt_zh: 这一篇专门复盘前端部分：主站首页如何从开源项目复刻并改造成自己的风格，Hexo + Butterfly 为什么是第一套博客，以及我为什么又单独做了一个成长博客。
excerpt_en: This post focuses on the frontend side: how the homepage was recreated from an open-source project, why Hexo + Butterfly became the first blog, and why I built a separate growth blog afterward.
words: 4300
reads: 152
minutes: 17
---

# 建站成长记录 02：把“能访问”变成“愿意停留”

上一篇我写的是基础设施，讲的是网站怎样先具备“能上线、能访问、能部署”的条件。  
但对一个真实的网站来说，这还远远不够。

用户真正感受到的网站，不是 DNS、不是服务器、不是 Actions，而是：

- 打开首页时第一眼看到的东西
- 点进博客时愿不愿意继续往下看
- 页面之间是不是有清晰的组织方式
- 这个网站有没有自己的气质

所以这一篇我想复盘的是：  
**当前这套网站的前端部分究竟是怎么长出来的。**

它不是一开始就完整规划好的，而是一步步扩展出来的：

1. 先有一个个人展示型首页
2. 再有一套真正能长期写文章的博客
3. 后面又补了一套专门记录“网站是怎么做出来的”的成长博客

最后，它们在同一个域名下形成了三套前端模块并存的结构。

## 先给出当前前端全景

从仓库结构上看，我现在的前端不是单一应用，而是三套前端并行：

| 模块 | 技术形态 | 当前定位 |
| --- | --- | --- |
| `main` | React + Vite | 主站首页、Bili 页面、留言板、AI 流量页 |
| `blog` | Hexo + Butterfly | 第一套正式博客，偏长期文章内容 |
| `build` | React + Vite + Markdown 静态生成 | 专门记录建站过程的成长博客 |

如果只从访问入口来看，也可以理解成：

| 路径 | 作用 |
| --- | --- |
| `https://taozhiyy.top/` | 主站 |
| `https://taozhiyy.top/blog/` | 主博客 |
| `https://taozhiyy.top/build/` | 成长博客 |

这三个入口并不是互相替代的关系，而是各自承担不同的内容任务。

## 为什么首页要先做好

如果只从“写文章”角度看，其实博客可以比首页更早完成。  
但我当时的想法是，一个个人网站不应该只是文章仓库，它最好先有一个**能够表达气质和方向的门面**。

首页的作用主要有三层：

1. 告诉别人“这是谁的站”
2. 告诉别人“这个站主要在做什么”
3. 给其他模块提供一个统一入口

如果没有首页，用户第一次进入站点时，体验更像是“误入一个技术目录”；  
而有了首页以后，整站才更像一个完整作品。

## 首页前端为什么会复刻 GitHub 上的开源项目

这部分我想写得坦诚一点。  
当前主站首页的前端，并不是从一张白纸开始完全手搓出来的，它的基础形态来自 GitHub 上的一个开源项目。

从仓库里的说明可以直接看出，这个首页是基于 **Adrian Hajdin** 的开源站点项目做了较大幅度的个性化改造。  
也就是说，我不是把一个现成模板原样照搬上线，而是先借用了一个完成度很高的开源首页骨架，再围绕自己的内容、审美和功能需求继续改。

我觉得这件事本身很正常，而且值得明确写出来，原因有三个。

### 1. 对个人项目来说，先借一个高质量骨架是很合理的

如果目标是尽快做出一个有展示力的首页，那么直接从成熟开源项目切入，往往比从零开始更现实。

这样做的好处是：

- 能快速得到完整页面节奏
- 能学习成熟的布局与动画组织
- 能把时间更多花在“改成自己的东西”上，而不是卡在最原始的页面搭骨架阶段

### 2. 真正关键的不是“是不是从零开始”，而是“你有没有完成自己的表达”

一个前端页面值不值得保留，最终看的是它有没有被真正改造成自己的站，而不是它最初是不是空白文件。

我现在这个首页之所以还保留下来，是因为它已经不只是原项目的外壳，而是被重新填进了自己的内容方向，比如：

- 视觉风格转向更偏二次元与个人表达
- 内容模块围绕自己的站点入口来组织
- 页面跳转会连接到博客、成长博客、Bili 页面等真实内容

### 3. 在成长记录里把来源说明白，比假装完全原创更真实

这篇文章既然是成长博客，我更愿意把真实过程讲清楚。  
我确实是从开源项目里吸收了成熟的前端表达方式，然后再往里面继续加自己的东西。

这不仅没有降低项目价值，反而是一次很好的实践：

- 学习如何阅读成熟前端结构
- 学习如何把公共骨架改造成个人项目
- 学习如何从“展示页面”慢慢过渡到“有真实业务功能的网站”

## 主站 `main` 现在承担什么

从当前代码来看，`main` 不只是一个漂亮的 landing page。  
它已经开始承载几个真正有交互性质的页面。

当前主站主要负责：

| 页面 | 当前作用 |
| --- | --- |
| `/` | 首页展示 |
| `/bili` | 番剧/创作者相关入口页 |
| `/guestbook` | 留言板 |
| `/ai-traffic` | AI 调用统计页 |
| `/login` | 打开 AI 助手登录入口的辅助路由 |

也就是说，`main` 这一层已经从“首页壳子”成长成了“整站交互前台入口”的角色。

这对我来说是一个很重要的变化，因为它说明首页不再只是装饰，而是开始承担功能导航作用。

## 为什么 Hexo + Butterfly 会成为第一套博客

如果首页解决的是“门面”问题，那么博客解决的就是“长期内容沉淀”问题。

我最早真正拿来持续写文章的博客，是 `Hexo + Butterfly` 这一套。  
这也是我在同一域名下挂出的第一套正式博客系统。

选择 `Hexo + Butterfly` 的原因其实很符合个人站的实际需求。

### 1. 写文章成本低

Hexo 的核心优势之一就是：

- Markdown 写作自然
- 目录结构清晰
- 生成静态页面方便
- 适合持续积累文章

对个人写作者来说，这种方式非常顺手。  
我不需要先解决复杂后台问题，就能先把“内容发出来”。

### 2. Butterfly 的视觉完成度高

Hexo 只是静态站生成器，真正让博客“像一个可读网站”的，是主题。  
我后来选择 Butterfly，很重要的一点就是它在个人博客场景里已经非常成熟：

- 导航和归档结构完整
- 文章阅读体验比较稳定
- 标签、分类、侧边栏等常见模块比较齐
- 可定制空间足够大

换句话说，它能让我比较快地得到一个“可长期使用”的博客外壳。

### 3. 适合先把内容积累起来

对我来说，博客最早的使命不是做出最复杂的系统，而是**先建立稳定写作习惯和内容出口**。

如果一开始就想把博客系统做得过于复杂，写作反而可能被打断。  
而 `Hexo + Butterfly` 的好处就是，它足够成熟、足够直接，能让我先把文章真正写起来。

## 为什么后来又单独做了一个成长博客

这一点是整个前端结构里我最想强调的一部分。

很多人会问：既然已经有 `Hexo + Butterfly` 博客了，为什么还要再做一个 `build`？

答案其实很简单：  
**因为“写普通文章”和“记录这个网站是怎么做出来的”是两种不同的表达任务。**

我后来越来越觉得，这个网站本身也值得被记录。  
它不只是一个承载内容的容器，它本身也是一个持续成长的作品。

于是，`build` 这套成长博客就出现了。

## `build` 和 `blog` 的定位区别

为了避免两套博客互相打架，我后来把它们的定位尽量分开。

| 模块 | 更适合写什么 |
| --- | --- |
| `blog` | 常规长文、个人表达、主题内容、项目文章 |
| `build` | 建站过程、结构复盘、功能记录、实现思路 |

也就是说：

- `blog` 更像“内容博客”
- `build` 更像“工程成长日志”

这样分开之后，好处很明显。

### 1. 内容边界更清楚

技术实现复盘不会把主博客的内容气质打散，主博客也不需要承担所有工程记录的任务。

### 2. 结构更适合长期扩写

随着网站越来越大，成长记录会越来越多。  
如果全塞到同一套博客里，后面很容易混乱。

### 3. 可以做不同的页面表达

`build` 用的是我自己控制更强的一套前端页面结构，它可以更偏“作品集 + 时间线 + 项目说明”的感觉，而不必完全服从传统博客主题的框架。

## `build` 这套成长博客为什么没有继续用 Hexo

这是一个很值得复盘的点。

如果只是为了继续写 Markdown 文章，继续沿用 Hexo 当然也可以。  
但我后来给 `build` 的目标，不只是“再来一个博客”，而是：

- 更自由的页面结构
- 更轻量的内容数据组织
- 更适合项目成长记录的展示方式
- 更方便中英双语切换

所以我最终让 `build` 走了一条不一样的路：

- 前端依然是 React + Vite
- 文章内容仍然可以用 Markdown 写
- 但内容会通过脚本生成到站点数据里

这个思路对我来说很合适，因为它兼顾了两件事：

1. 保留 Markdown 写作体验
2. 保留前端页面高度可控的自由度

## `build` 的前端思路：不是纯博客，更像“项目说明站”

我很喜欢 `build` 现在的一点，是它虽然也有文章页、归档页、列表页，但整体感觉并不完全像传统博客。

它更像一个围绕项目成长过程组织的内容站，通常会包括：

- 文章列表
- 时间线归档
- 成长记录
- 相册或展示页
- 链接页
- 留言页

它和主博客的差别，不只在内容主题，也在页面气质上：

- `blog` 偏经典博客阅读
- `build` 偏项目展示与成长记录

这也是为什么我觉得它值得独立存在。

## 为什么这套成长博客要支持中英文切换

`build` 这一块还有一个我很喜欢的点，就是它不是只面向中文内容。  
它从结构上已经支持中英文切换，这对于“成长记录”这种类型尤其有意义。

我之所以愿意给它做双语支持，主要有三点原因。

### 1. 让内容表达更完整

很多工程类记录其实很适合双语组织。  
中文写起来更自然，英文则更方便用另一套语境重新概括结构与思路。

### 2. 让站点本身更像一个长期作品

双语不是为了炫技，而是为了让这个站更像一个会持续迭代的个人项目。  
一旦站点成长记录本身也具备中英文表达能力，它的可扩展性就更高。

### 3. 方便把“过程”整理成更通用的经验

成长博客并不只是给当下自己看的，它也可能是未来复盘或分享时的基础材料。  
双语结构会迫使我把内容组织得更清楚。

## 当前三套前端模块的关系

现在回头看，这三套前端并不是重复建设，而是分工逐渐清晰的结果。

| 模块 | 面向什么 | 关键词 |
| --- | --- | --- |
| `main` | 第一次访问者与日常交互 | 首页、入口、展示、交互 |
| `blog` | 长期文章阅读 | 写作、阅读、归档 |
| `build` | 建站过程与项目记录 | 成长、结构、复盘 |

如果把它们都压进一个系统里，确实也不是绝对不行；  
但拆开之后，表达效率反而更高。

## 截至目前，前端已经走到了哪一步

虽然这篇文章是按搭建顺序回放，但我还是想加一段“截至目前”的状态总结。

截至目前，我的网站前端已经不再只是一个单页面试验，而是形成了比较完整的内容与交互层：

- 有主站首页
- 有第一套正式博客
- 有单独的成长博客
- 有留言板页面
- 有 AI 流量统计页
- 有 Bili 相关入口页
- 有全站统一 AI 助手入口

这说明前端层已经从“展示页面”逐步进入“系统页面”阶段。

## 我对这一阶段最大的感受

如果让我总结这一阶段前端实现最重要的一点，不是“我用了什么技术栈”，而是：

> 我开始把不同内容任务拆给不同页面系统，而不是强迫一个系统承担所有事情。

这带来的变化很大：

- 首页可以更重视觉表达
- 主博客可以更重阅读体验
- 成长博客可以更重结构记录

每一层都不用再为了兼顾所有目标而变得臃肿。

## 下一篇写什么

有了基础设施，也有了几套前端页面之后，网站就不再只是“看起来像一个站”。  
接下来真正决定它是否有持续交互能力的，就是后端部分。

下一篇我会继续写当前已经落地的后端能力，包括：

- `/api/` 现在有哪些接口
- 留言板是怎么支撑起来的
- 登录、注册和站长二次验证怎么做
- AI 助手和额度控制是怎么工作的
- GitHub Actions 又是怎样把这些服务自动部署到 UCloud 的

系列上一篇：

- [建站成长记录 01｜前提准备：Cloudflare、UCloud 与 NameSilo 的基础搭建](https://taozhiyy.top/build/post/site-growth-01-infra-prep)

系列下一篇：

- [建站成长记录 03｜后端实现：接口、登录、留言板与 AI 助手](https://taozhiyy.top/build/post/site-growth-03-backend-services)

---en---

# Site Growth Log 02: Turning “Reachable” into “Worth Staying For”

The previous post focused on infrastructure: how the site became deployable, reachable, and maintainable.  
But for a real website, that is still not enough.

What people actually experience is not DNS, not the server, and not GitHub Actions. What they really notice is:

- what they see when they first open the homepage
- whether they want to keep reading once they enter the blog
- whether the pages feel organized
- whether the site has a recognizable personality

So this post is my frontend retrospective:  
**how the current frontend structure of the site actually came into existence.**

It was not born as one complete plan. It grew in stages:

1. first a personal showcase homepage
2. then a real blog system for long-form writing
3. then a separate growth blog for documenting how the site itself was built

Over time, those became three frontend modules under the same domain.

## The current frontend landscape

From the repository structure, the site now has three parallel frontend modules rather than one monolithic app:

| Module | Tech form | Current role |
| --- | --- | --- |
| `main` | React + Vite | Homepage, Bili page, guestbook, AI stats page |
| `blog` | Hexo + Butterfly | First formal blog system |
| `build` | React + Vite + Markdown-driven static generation | Dedicated growth blog |

From the user-facing side, that becomes:

| Path | Purpose |
| --- | --- |
| `https://taozhiyy.top/` | Main site |
| `https://taozhiyy.top/blog/` | Main blog |
| `https://taozhiyy.top/build/` | Growth blog |

These modules do not replace each other. They serve different content goals.

## Why the homepage mattered first

If I only cared about publishing articles, the blog could have come first.  
But my thinking was that a personal site should not feel like a storage box for posts. It should first have a visible front door.

The homepage needed to do three things:

1. say whose site this is
2. say what kind of work the site contains
3. give users a unified starting point for everything else

Without a homepage, the first experience feels like stumbling into a directory.  
With one, the whole site starts to feel like a deliberate project.

## Why the homepage frontend was recreated from an open-source GitHub project

I want to be direct about this part.  
The current homepage frontend did not begin from a totally blank file. Its initial shape came from an open-source project on GitHub.

The repository notes already make that clear: the homepage was heavily customized from an open-source website project by **Adrian Hajdin**.  
So I did not simply drop a template online unchanged. I started from a strong open-source visual structure and then adapted it around my own content, tone, and goals.

I think that is both normal and worth stating openly, for three reasons.

### 1. For a personal project, borrowing a strong skeleton is practical

If the goal is to quickly build a homepage with strong visual presence, starting from a mature open-source project is often far more realistic than starting from nothing.

It helps because:

- the page rhythm already exists
- the layout and animation structure can be studied directly
- more time can be spent on customization instead of fighting the earliest page-building stage

### 2. What matters is not whether it started from zero, but whether it became yours

What makes a page worth keeping is not whether it began as an empty file.  
It is whether it was transformed into something that genuinely expresses your project.

My homepage remains valuable because it has already moved beyond the original shell:

- the visual tone shifted toward a more personal and anime-inspired direction
- the content modules were reorganized around my actual site structure
- the homepage now links into the real living parts of the project

### 3. In a growth log, honesty is more useful than pretending everything was original from the first line

This series is a build diary. I would rather describe the real path clearly.  
I did learn from a mature open-source frontend project, and then continued shaping it into my own site.

That does not reduce the value of the work. If anything, it reflects a good real-world process:

- learning from a strong structure
- adapting a general skeleton into a personal site
- evolving a showcase page into a site with real interaction and systems behind it

## What the `main` frontend handles now

At this point, `main` is no longer just a visual landing page.  
It already carries several interactive pages.

Its current responsibilities include:

| Page | Role |
| --- | --- |
| `/` | Main homepage |
| `/bili` | Bili-related entry page |
| `/guestbook` | Guestbook |
| `/ai-traffic` | AI usage statistics |
| `/login` | Helper route for opening the AI login panel |

So `main` has already grown from a landing shell into the primary interactive frontend entry for the site.

That matters because it means the homepage is no longer just decorative. It has started to act as a functional navigation layer.

## Why Hexo + Butterfly became the first formal blog

If the homepage solves the front-door problem, the blog solves the long-term content problem.

My first real, sustained blog system was `Hexo + Butterfly`.  
This was the first formal blog I exposed under the domain.

The reasons fit the practical needs of a personal site quite well.

### 1. Low writing friction

One major advantage of Hexo is that it makes writing straightforward:

- Markdown feels natural
- the content structure is clear
- static generation is simple
- it is easy to maintain over time

For a personal writing workflow, that matters a lot.  
I did not need to build a complex backend before I could start publishing real articles.

### 2. Butterfly has a strong level of visual maturity

Hexo is the generator. The theme is what turns it into a readable site.  
I chose Butterfly because it already feels mature for personal blogging:

- navigation and archive structures are complete
- article reading experience is stable
- tags, categories, sidebars, and other common blog patterns are already there
- it still leaves room for customization

So it gave me a long-term blog shell much faster.

### 3. It let me start accumulating content immediately

At the beginning, my main goal was not building the most complex blog system possible.  
It was building a stable writing habit and a real publishing outlet.

If the system itself becomes too complicated too early, writing gets delayed.  
`Hexo + Butterfly` avoided that problem for me.

## Why I later built a separate growth blog

This is the part I care most about in the frontend story.

A natural question is: if I already had a `Hexo + Butterfly` blog, why create another module called `build`?

The answer is simple:  
**writing ordinary articles and documenting how the site itself is being built are two different kinds of expression.**

At some point I realized the website itself was worth documenting.  
It was no longer just a container for content. It had become a growing work in its own right.

That is why the `build` growth blog exists.

## The role difference between `build` and `blog`

To keep the two systems from overlapping too much, I gradually separated their roles.

| Module | Best suited for |
| --- | --- |
| `blog` | regular long-form writing, personal expression, topic-focused posts |
| `build` | build process logs, architecture retrospectives, implementation notes |

In short:

- `blog` is the content blog
- `build` is the engineering growth log

That separation brought several benefits.

### 1. Clearer content boundaries

Implementation retrospectives no longer dilute the tone of the main blog, and the main blog does not need to carry every engineering note.

### 2. Better long-term scalability

As the website grows, the amount of build documentation grows too.  
Putting everything in one place would eventually become messy.

### 3. Different frontend expression is possible

`build` uses a frontend structure with more direct control. It can feel more like a project showcase and growth archive than a conventional blog theme.

## Why `build` did not simply reuse Hexo

This is an important design decision to reflect on.

If the only goal were to keep writing Markdown posts, continuing with Hexo would absolutely have worked.  
But I wanted `build` to be more than “another blog.”

I wanted:

- freer page structure
- lighter content data organization
- presentation better suited for project growth records
- easier bilingual switching

So `build` ended up on a different path:

- the frontend is still React + Vite
- articles are still written in Markdown
- but a script turns content into site data for the frontend

That approach works very well for me because it preserves both:

1. the comfort of Markdown writing
2. the freedom of a highly controlled frontend

## The design idea behind `build`: not just a blog, but a project explanation site

What I like about `build` is that although it has article pages, archives, and lists, it does not feel exactly like a traditional blog.

It feels closer to a content site built around project growth:

- article list
- archive timeline
- build logs
- gallery or showcase pages
- links page
- message page

The difference from the main blog is not only the subject matter. It is also the page identity:

- `blog` leans toward classic reading
- `build` leans toward project presentation and process documentation

That is why I think it deserves to exist as its own frontend module.

## Why the growth blog supports both Chinese and English

Another part I really like about `build` is that it was not designed only for Chinese content.  
Its structure already supports bilingual switching, which feels especially meaningful for a build log.

I wanted bilingual support for three reasons.

### 1. It makes the writing more complete

Engineering retrospectives often benefit from bilingual framing.  
Chinese allows a more natural narrative flow for me, while English helps summarize the structure in another way.

### 2. It makes the project feel more like a long-term work

The goal is not to show off. It is to make the site feel like an evolving personal project with room to grow.

### 3. It helps me organize process knowledge more clearly

The growth blog is not only for the current version of myself. It may later become reusable material for sharing or reflection.  
Bilingual structure forces clearer thinking.

## How the three frontend modules relate now

Looking back, these three frontend systems are not redundant. They are the result of clearer responsibilities.

| Module | Who it serves | Keywords |
| --- | --- | --- |
| `main` | first-time visitors and interactive entry | homepage, showcase, entry, interaction |
| `blog` | long-form article reading | writing, reading, archives |
| `build` | documenting the site and the project itself | growth, structure, retrospective |

Could they all have been forced into one system? Maybe.  
But separating them made each one better at its own job.

## Where the frontend stands right now

Even though this post retells the frontend in sequence, I also want to summarize the current state.

At this point, the frontend is no longer a one-page experiment. It has become a layered content and interaction surface:

- a main homepage
- a first formal blog
- a dedicated growth blog
- a guestbook page
- an AI usage statistics page
- a Bili-related entry page
- a global AI assistant entry

That means the frontend has already moved from “display pages” into “system pages.”

## My biggest takeaway from this stage

If I had to summarize the most important frontend lesson from this stage, it would not be about the exact framework.

It would be this:

> I stopped forcing one frontend system to solve every content problem, and instead gave different roles to different page systems.

That changed everything:

- the homepage could focus on visual identity
- the main blog could focus on reading
- the growth blog could focus on structure and documentation

Each layer became lighter and more effective.

## What comes next

Once the infrastructure exists and the frontend layers are visible, the site finally starts looking real.  
But the part that decides whether the project can support deeper interaction is the backend.

In the next post, I will cover the backend features that are already live, including:

- what endpoints currently exist under `/api/`
- how the guestbook is supported
- how login, registration, and owner verification work
- how the AI assistant and quota control work
- how GitHub Actions deploys the whole thing to UCloud

Previous in the series:

- [Site Growth Log 01 | Foundation Setup with Cloudflare, UCloud, and NameSilo](https://taozhiyy.top/build/post/site-growth-01-infra-prep)

Next in the series:

- [Site Growth Log 03 | Backend Implementation: APIs, Login, Guestbook, and AI Assistant](https://taozhiyy.top/build/post/site-growth-03-backend-services)
