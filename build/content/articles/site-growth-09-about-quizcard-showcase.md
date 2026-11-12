---
date: 2026-06-24
slug: site-growth-09-about-quizcard-showcase
title_zh: 建站成长记录 09｜关于本站页、项目集与 AI QuizCard 成品页上线
title_en: Site Growth Log 09 | About Page, Project Showcase, and AI QuizCard Launch
excerpt_zh: 这篇记录“关于我 / 关于本站”页面从本地预览走进主站的过程：它成为顶部导航里的正式页面，图片资源统一上传到腾讯 COS，项目集完成部署，第一个项目 AI QuizCard 指向真正的成品页，并补上访客可直接试玩的种子题、卡片布局和练习报告滚动修复。
excerpt_en: This post records how the About page moved from a local preview into the main site: it became a first-class navigation page, images were served from Tencent COS, the project showcase was deployed, the first project now opens the real AI QuizCard page, and the visitor app gained seed data, layout fixes, and a safer practice-report scroll flow.
words: 4300
reads: 14
minutes: 14
---

# 建站成长记录 09：把“关于本站”变成真正能访问、能游玩、能继续扩展的页面

这次更新表面上是一个页面上线，实际更像是把一组散落的设计稿、预览页和项目文件收进主站秩序里。

之前“关于我 / 关于本站”已经有了一份很完整的本地预览页，但它还停留在 `file://` 和临时预览阶段。项目卡片、游戏卡片、工具图标、图片资源和第一个项目 AI QuizCard 都有了雏形，却还没有成为网站访问者能稳定打开的线上页面。

这次的目标很明确：

- 主站顶部导航要有一个正式的“关于我”入口
- `/about` 要加载这份关于页面，而不是只依赖本地预览
- 关于页里的图片要走腾讯 COS，避免本地路径和临时资源失效
- 项目集要像子页面一样部署好，而不是打开本地地址
- 第一个项目 AI QuizCard 要跳转到真正的成品展示页
- 游客打开 QuizCard 后也能直接试玩，而不是卡在“载入中”
- 移动端和窄屏下，卡片、横向区域和练习报告不能互相覆盖

这也是一次很典型的个人站迭代：先有一个好看的预览，再把它变成可维护、可访问、可继续长大的正式页面。

## 关于页进入主站导航

主站现在在顶部导航中加入了“关于我”，对应路由是：

| 入口 | 路由 | 说明 |
| --- | --- | --- |
| 顶部导航“关于我” | `/about` | 正式的关于页入口 |
| 项目子页面 | `/about/projects/:projectId` | 用于承接项目集里的项目 |
| 第一个项目 | `/showcase/quizcard.html` | AI QuizCard 成品展示页 |

这里没有把关于页做成一个孤立的静态文件，而是让它进入 React 主站路由。这样它可以使用主站的导航、移动端菜单、页脚主题和部署流程，也不会和首页、相册、碎语、数据中心这些页面割裂。

实际实现里，`/about` 会读取 `main/public/about-preview.html`，再把其中的样式和内容放进一个隔离的 Shadow DOM 容器。这样做有两个好处：

1. 预览页原本完整的 HTML/CSS 可以被保留下来
2. 它不会把全局样式污染到主站其他页面

这相当于给“关于本站”套了一层稳定的主站外壳：页面内容仍然保持独立视觉，但路由和访问方式已经属于主站。

## 去掉预览痕迹

把本地预览页写进主站时，有一个很容易忽略的问题：预览页上那些只给自己看的提示文字不能出现在正式页面里。

所以左上角“预览专用”之类的提示被移除。正式页面打开后，访问者不应该感到自己进入了一个临时测试文件，而应该感到这是网站的一部分。

这个细节很小，但对页面气质很重要。一个个人站可以保留手作感和成长痕迹，但不应该把调试脚手架留在用户面前。

## 图片资源改走腾讯 COS

关于页的图片也做了统一处理。

本地预览时，图片可以直接从电脑路径读取；但只要写进主站，这些路径就会失效。于是这次把关于页需要的图片上传到腾讯 COS，再在页面中引用稳定的公网链接。

当前规则是：

| 资源类型 | 处理方式 |
| --- | --- |
| 关于页人物、兴趣和游戏图片 | 上传到腾讯 COS 后引用公网 URL |
| 主站导航和页面资源 | 继续使用已有 COS 资源封装 |
| 项目展示页静态资源 | 放入 `main/public/showcase/` 和 `main/public/web/` 随主站部署 |

这样主站发布后，不管访问者从手机、桌面还是外部链接打开页面，都不会依赖我的本地文件。

这也延续了前面 AI 生图和相册的资源策略：公开页面展示的图片应该是稳定资源，不应该绑在某台电脑、某次预览或某个临时模型链接上。

## 关于页卡片调整

这次关于页也做了不少视觉和交互整理。

最明显的是游戏卡片和项目卡片。之前游戏卡片在某些尺寸下会被缩放得不协调，项目卡片也比本地预览显得拥挤。现在页面宽度、项目网格和卡片尺寸都重新拉开：

| 区域 | 调整 |
| --- | --- |
| 项目集 | 宽屏下增加列数和留白，让卡片不挤在一起 |
| 游戏卡片 | 竖向卡片保持稳定尺寸，不再为了适配而过度缩放 |
| 游戏区域 | 直接展示所有常驻游戏，不再依赖底部横向拉条 |
| 模块说明文字 | 从过于居中的位置往边缘靠，页面呼吸感更自然 |
| 技术工具图标 | 自动补齐轮播内容，避免滚动到一半出现空白 |

这些都是“整体协调”的问题，不是单点样式问题。

一个页面里如果每个模块都好看，但模块之间节奏不对，就会让人感觉散。相反，卡片尺寸、文字位置、滚动节奏、留白和图片比例一起稳定后，页面会变得更像一个完整作品。

## 文案也跟着变轻一点

这次还改了关于页里几处文案。

比如“目前一份在手，更多在路上。每一格都会慢慢被填满。”这种句子，意思是对的，但放在大模块里有点像说明牌。调整后的方向更贴近个人站：少一点汇报感，多一点“正在长出来”的感觉。

“我玩的游戏最近常驻这几款”和“不务正业小档案”这类文案也重新收了一下。关于页不是简历，也不是功能说明书，它更像一个可以被浏览的个人小档案。文案要有性格，但不能抢过视觉结构。

这类文字调整看起来不像工程，却会影响整个页面的可信度。个人站最怕的是“模块很多，但人不在里面”。这次我希望文字能更靠近真实的自己，而不是像在给页面做注释。

## 项目集不再指向本地预览

项目集这次从“页面里的一个展示模块”变成了真正可访问的入口。

之前有一版问题是：点击项目后打开的是本地预览，甚至可能跳到 `127.0.0.1` 或临时文件。对开发时自己看没问题，但对访客来说这就是断路。

现在第一个项目 AI QuizCard 的跳转关系变成：

| 页面 | 行为 |
| --- | --- |
| `/about` 项目卡片 | 点击后进入 `/showcase/quizcard.html` |
| `/about/projects/quizcard` | 自动跳转到 `/showcase/quizcard.html` |
| `/showcase/quizcard.html` | 展示 AI QuizCard 成品页和入口 |
| `/web/index.html` 等静态页 | 提供访客直接试玩的 QuizCard 小应用 |

这样“项目集是关于页的子内容，项目又有自己的成品页”这条关系就顺了。

关于页负责介绍我和我的作品，项目展示页负责让访问者真正体验作品，而不是只看一张卡片。

## AI QuizCard 的第一个成品页

AI QuizCard 是这次项目集里的第一个项目。

它的定位不是一个只放截图的展示页，而是一个可以让访客直接打开、试用、理解流程的小工具。页面包含成品介绍、移动端页面入口、访客试玩入口和静态资源。

当前部署结构大致是：

| 部分 | 作用 |
| --- | --- |
| `main/public/showcase/quizcard.html` | 项目成品展示页 |
| `main/public/showcase/assets/` | 展示页样式和脚本 |
| `main/public/showcase/mp-pages/` | 小程序风格的页面预览 |
| `main/public/web/` | 访客可以直接玩的静态 QuizCard 应用 |
| `main/public/web/assets/seed.js` | 默认种子题库 |

这次最重要的调整是：访客不需要后端数据也能进入练习。

如果一个项目展示页打开后一直显示“载入中”，访问者不会去想“是不是还没接后端”。他们只会觉得它坏了。所以我先补了几组种子题，让页面在没有用户导入数据时也能完整跑通。

以后用户可以自己导入题目，但默认状态必须可玩。

## 访客直接玩

QuizCard 静态访客版现在带有几个默认题组：

- 神经科学基础
- 英语词汇
- 物理概念

这些不是为了替代用户自己的题库，而是为了保证第一次打开时有内容可练。一个工具如果需要用户先准备一堆数据才能体验，它的进入门槛会非常高。

现在访客打开后可以直接：

1. 选择默认题组
2. 开始练习
3. 作答
4. 查看练习报告
5. 后续再导入自己的题目

这让 QuizCard 从“展示一个我做过的东西”更接近“让你马上摸到它怎么工作”。

## 练习报告滚动修复

最后一个比较关键的体验问题出现在练习报告页。

窄屏下，报告主信息卡原本会 sticky 在顶部，错题区往上滚动时就可能覆盖主界面。这个交互在桌面端看起来像固定摘要，但在手机上会变成遮挡。

这次调整为：

| 视口 | 报告摘要行为 |
| --- | --- |
| 窄屏 / 移动端 | 跟随页面一起滚动，不覆盖错题 |
| 宽屏 | 保留 sticky 摘要，方便边看报告边浏览错题 |

这个修复背后其实是一个很通用的原则：桌面端好用的固定信息，在手机上可能就是遮挡。响应式不是把同一个布局缩小，而是让每个屏幕尺寸下的阅读顺序都成立。

## 为什么这次要写进 build

这次更新跨了很多层：

- 主站导航
- React 路由
- Shadow DOM 集成静态预览页
- 腾讯 COS 图片资源
- 关于页视觉与文案
- 项目集部署
- QuizCard 成品展示页
- QuizCard 访客静态应用
- 种子题库
- 移动端报告布局修复
- README 和发布前敏感信息检查

如果只看最终页面，很容易以为这是一次“调样式”。但真正的变化是：这页从我的本地文件，变成了主站的一部分。

这也是 build 记录存在的意义。它不是只写“大功能”，也记录那些让网站从预览走向上线的中间环节。

## 发布前的安全检查

这次发布前也专门做了敏感信息扫描。

扫描范围分成两层：

| 范围 | 目的 |
| --- | --- |
| 当前已跟踪文件 | 检查即将发布的代码和文档里有没有明文密钥 |
| 远端分支和 tag | 检查 GitHub 上可达的历史里有没有高置信密钥格式 |

重点检查了 OpenAI、GitHub token、AWS、腾讯云 COS、阿里云、Google API、Slack token、JWT 和私钥块等常见格式。

当前可发布范围没有发现这类真实私钥。仓库里保留的是环境变量名、GitHub Actions Secrets 引用和公开搜索配置。后续仍然要保持这个规则：真实密码、API Key、COS Secret、服务器凭据、数据库和 `.env` 不进仓库。

## 接下来的计划

关于页现在已经进入主站，项目集也有了第一个能打开的成品项目。

接下来更适合继续补三件事：

| 计划 | 目标 |
| --- | --- |
| 继续填项目集 | 把后续项目一格一格补上，不让卡片只停留在占位状态 |
| 强化 QuizCard 导入体验 | 让用户导入题目、解析内容、保存题库的路径更顺 |
| 统一项目展示规范 | 后续每个项目都能有介绍页、体验入口、截图和版本记录 |

我很喜欢这次更新的一个点：它没有只追求“看起来像一个关于页”，而是开始让关于页承担真实导航和作品入口的职责。

个人站最终不应该只有漂亮首页，也应该有能让别人继续往里走的路径。关于页就是其中一条。

---en---

# Site Growth Log 09: Turning the About Page Into a Real Site Page

This update looks like a page launch on the surface, but it is really about moving a complete preview into the site system.

The About page already existed as a local preview. It had project cards, game cards, tool marquees, image assets, and the first project, AI QuizCard. But it still lived in the preview stage: local files, temporary links, and project entries that were not yet fully deployed.

The goals were clear:

- add About as a first-class navigation item
- make `/about` load the About page inside the main site
- serve About page images from Tencent COS
- deploy the project showcase instead of opening local preview URLs
- make the first project open the real AI QuizCard showcase page
- let visitors play QuizCard immediately, even without backend data
- fix mobile and narrow-screen layout issues around cards and reports

This is a familiar personal-site step: first make a good preview, then turn it into something stable, reachable, and ready to grow.

## About enters the main navigation

The main site now has a top-navigation entry named About, using the route:

| Entry | Route | Purpose |
| --- | --- | --- |
| About nav item | `/about` | Main About page |
| Project child route | `/about/projects/:projectId` | Project-level handoff route |
| First project | `/showcase/quizcard.html` | AI QuizCard showcase |

The About page is not left as an isolated static file. It is mounted through the React site route, so it can share the main navigation, mobile drawer, page theme, footer behavior, and deployment pipeline.

Under the hood, `/about` fetches `main/public/about-preview.html`, scopes its CSS, and renders it inside a Shadow DOM host. That keeps the preview page visually independent without letting its styles leak into the rest of the site.

The result is a stable wrapper around the About page: the visual language stays independent, while the route and access path belong to the main site.

## Removing preview traces

When a local preview becomes a production page, small preview-only text needs to disappear.

The upper-left preview note was removed. A visitor should not feel as if they opened a temporary test file. They should feel that this page belongs to the site.

That sounds small, but it changes the tone. A personal website can keep its handmade feeling, but debug scaffolding should not be visible to visitors.

## Images moved to Tencent COS

The About page image assets were also stabilized.

Local image paths work during preview, but they break once the page is deployed. So the About page images were uploaded to Tencent COS and referenced through public URLs.

Current asset handling is:

| Asset type | Handling |
| --- | --- |
| About page people, interest, and game images | uploaded to Tencent COS and referenced by public URL |
| Main-site navigation and shared assets | continue using the existing COS asset helper |
| QuizCard static showcase assets | shipped under `main/public/showcase/` and `main/public/web/` |

This means the page no longer depends on a local machine, a temporary preview, or a model-generated temporary URL.

It follows the same direction as the AI image and gallery work: public pages should use stable public assets.

## Card and layout polish

This pass also tightened a lot of visual details.

The game cards and project cards were the most visible areas. Some game cards were being scaled too aggressively, the project grid felt tighter than the preview, and the tool marquee could briefly show blank space during continuous movement.

The updates were:

| Area | Change |
| --- | --- |
| Project showcase | wider layout and roomier grid across desktop breakpoints |
| Game cards | stable vertical card dimensions instead of over-scaling |
| Game section | show all regular games directly instead of relying on a horizontal scrollbar |
| Module intro copy | moved away from the visual center toward the edge for a better rhythm |
| Tool marquee | duplicates content dynamically so the movement does not expose empty gaps |

These are not isolated style tweaks. They are coordination work.

A page can have many nice modules and still feel scattered if spacing, motion, text position, and image scale do not agree with each other. Once those pieces settle, the page starts to feel like one composition.

## Copywriting cleanup

Several About page text snippets were also revised.

Sentences like “currently one in hand, more on the way” had the right meaning, but they felt too much like explanatory labels inside larger modules. The new direction is lighter: less report-like, more like something still growing.

Labels around regular games and personal side notes were also tightened. The About page is not a resume and not a feature manual. It is closer to a browseable personal file. The copy can have personality, but it should not fight the visual structure.

This matters because personal pages can easily become a pile of modules without a person inside them. The words should make the page feel more like me, not more like a UI annotation layer.

## Project showcase no longer opens local previews

The project showcase is now a real access path, not just a section on the About page.

The earlier problem was that clicking a project could open a local preview or a `127.0.0.1` address. That is fine during development, but it is a dead end for visitors.

The first project now follows this route:

| Page | Behavior |
| --- | --- |
| `/about` project card | opens `/showcase/quizcard.html` |
| `/about/projects/quizcard` | redirects to `/showcase/quizcard.html` |
| `/showcase/quizcard.html` | shows the AI QuizCard product page and entry points |
| `/web/index.html` and related files | provide the visitor-playable QuizCard static app |

This makes the hierarchy clearer: About introduces me and my projects; project pages let visitors experience the work.

## AI QuizCard as the first launched project

AI QuizCard is the first project in this showcase.

It is not just a screenshot page. It is a small tool that visitors can open, try, and understand through interaction.

The deployed structure is roughly:

| Part | Purpose |
| --- | --- |
| `main/public/showcase/quizcard.html` | product showcase page |
| `main/public/showcase/assets/` | showcase styles and scripts |
| `main/public/showcase/mp-pages/` | mini-program-style page previews |
| `main/public/web/` | visitor-playable static QuizCard app |
| `main/public/web/assets/seed.js` | default seed decks |

The most important change is that visitors can now use it without backend data.

If a showcase page stays stuck on “Loading,” visitors will not think “maybe the backend is not connected yet.” They will think it is broken. So the app now ships with seed decks that let the full practice loop run immediately.

Users can still import their own material later, but the default state must be playable.

## Visitor play mode

The static QuizCard visitor app now includes a few seed decks:

- basic neuroscience
- English vocabulary
- physics concepts

These are not meant to replace user-owned decks. They exist so the first visit has something real to practice.

Visitors can now:

1. select a seed deck
2. start a practice session
3. answer cards
4. view the report
5. import their own content later

This shifts QuizCard from “look at something I made” toward “touch the workflow immediately.”

## Practice report scroll fix

The last important UX issue was on the practice report page.

On narrow screens, the report summary card was sticky, and the wrong-answer review section could scroll underneath it. On desktop that sticky summary is useful. On mobile it becomes an obstruction.

The behavior is now split by viewport:

| Viewport | Summary behavior |
| --- | --- |
| Narrow / mobile | scrolls with the page, so it does not cover the review area |
| Wide | stays sticky, so the summary remains visible while reviewing mistakes |

The general lesson is simple: a fixed desktop summary can become a mobile overlay bug. Responsive design is not shrinking one layout. It is preserving the reading order for each screen size.

## Why this belongs in the build log

This update touches many layers:

- main-site navigation
- React routing
- Shadow DOM integration for a static preview
- Tencent COS image assets
- About page visual and copy polish
- project showcase deployment
- AI QuizCard showcase page
- QuizCard visitor static app
- seed decks
- mobile report layout
- README and pre-release secret scanning

If you only look at the final page, it may look like a style pass. But the real change is that the page moved from my local file system into the public site.

That is exactly what the build log is for. It records not only the big features, but also the work that turns previews into shipped pages.

## Pre-release security check

Before release, I also ran a focused secret scan.

The scan had two layers:

| Scope | Purpose |
| --- | --- |
| current tracked files | check the code and docs about to be released |
| remote branches and tags | check GitHub-reachable history for high-confidence secret formats |

The scan focused on common formats for OpenAI keys, GitHub tokens, AWS keys, Tencent COS identifiers, Alibaba Cloud keys, Google API keys, Slack tokens, JWTs, and private-key blocks.

The publishable refs did not contain real private secrets. The repository keeps environment variable names, GitHub Actions Secrets references, and public search configuration only. The rule remains: passwords, API keys, COS secrets, server credentials, databases, and `.env` files do not go into the repository.

## Next steps

The About page is now part of the main site, and the project showcase has its first project with a real playable entry.

The next steps are:

| Plan | Goal |
| --- | --- |
| Fill the project showcase | add more real projects instead of leaving cards as placeholders |
| Improve QuizCard import flow | make parsing, saving, and practicing user content smoother |
| Standardize project pages | give future projects a consistent intro, experience link, screenshots, and version notes |

What I like most about this update is that it did not only make the About page look complete. It gave the page a real job: navigation, identity, and project entry.

A personal site should not only have a beautiful homepage. It should also give visitors paths to keep walking inward. The About page is now one of those paths.
