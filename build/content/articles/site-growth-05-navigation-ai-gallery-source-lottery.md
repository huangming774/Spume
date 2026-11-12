---
date: 2026-06-05
slug: site-growth-05-navigation-ai-gallery-source-lottery
title_zh: 建站成长记录 05｜原创化继续推进：导航、AI 检测、相册迁移与 Source Lottery
title_en: Site Growth Log 05 | Continuing Originalization: Navigation, AI Health Checks, Gallery Migration, and Source Lottery
excerpt_zh: 这一篇记录今天围绕首页继续推进原创化的一轮更新：导航栏回归更清晰的入口逻辑，AI 固定检测脚本让服务状态可持续观察，相册从 blog 与 build 迁移到主站 Gallery，而 Source Lottery 则把原本普通的参考来源卡片重做成更有个人表达的老虎机抽奖交互。
excerpt_en: This post records today's next round of homepage originalization: a clearer navigation model, a lightweight AI health-check scheduler, gallery migration into the main site, and most importantly, Source Lottery, which transforms the old source-reference card into a personal arcade-style slot interaction.
words: 7200
reads: 88
minutes: 28
---

# 建站成长记录 05：原创化不是一次改完，而是每天往自己的方向推进

上一篇成长记录写的是首页原创化的一轮大改：Hero 指南针、About 展示、Features 档案书、Story 信纸式叙事、Footer 学习声明，以及移动端适配。

今天这一轮更新更像是把那次改造继续往深处推。  
它不只是继续改视觉，也开始整理网站的入口、功能边界和访客交互方式。

这次主要做了四件事：

- 导航栏修正
- AI 固定检测脚本
- 相册入口统一迁移到主站 Gallery
- 把“本站参考来源”改造成 `Source Lottery` 小型老虎机抽奖交互

其中最重要的是最后一项。  
因为它不是简单把一个卡片换成另一个卡片，而是把“说明来源”这件原本很静态、很说明书式的内容，改成了一个访客可以参与的小互动。

这也是我现在理解的原创化：

> 原创化不是把参考项目的痕迹一夜之间全部抹掉，而是不断把页面里的每一个位置重新问一遍：这里能不能更像我的网站？

## 今天这轮更新为什么重要

如果说上一轮原创化主要解决的是“首页看起来像不像我”，那今天这轮更偏向解决另一个问题：

> 这个网站的入口、内容和说明方式，是不是也开始拥有自己的组织逻辑？

一个个人网站不能只靠几个漂亮动画成立。  
它还需要让访客知道：

- 从哪里回到首页
- 去哪里看文章
- 去哪里看相册
- 留言入口在哪里
- AI 助手是否稳定
- 参考来源与原创改造之间的关系是什么

这些事情听起来没有 Hero 动画那么显眼，但它们决定了网站是不是一个真正能长期维护的整体。

所以今天的更新不是“再加一点装饰”，而是一次偏结构化的整理。

## 一、导航栏修正：让 HOME 真正成为回到主页的入口

这次先处理的是导航栏。

之前首页里有 `about`、`contact`、`end` 这样的锚点式导航。  
这种设计在单页展示站里很常见，但我的网站现在已经不是单纯的展示页了。

当前主站已经逐渐有了这些入口：

| 入口 | 当前作用 |
| --- | --- |
| `/` | 主站首页，也是整个网站的第一入口 |
| `/gallery` | 统一相册入口 |
| `/guestbook` | 主站留言墙 |
| `/ai-traffic` | AI 使用统计 |
| `/bili` | Bili 相关页面 |
| `/blog/` | Hexo + Butterfly 博客 |
| `/build/` | 成长博客 |

在这种情况下，导航栏最需要的不是继续把首页拆成几个锚点，而是让访客在任何子页面都能快速回到主站。

所以这次把原先首页里没有实际跳转意义的 `about`、`contact`、`end` 统一调整为 `HOME`。

现在的逻辑更清晰：

- 在首页点击 `HOME`，回到 Hero 区域
- 在子页面点击 `HOME`，返回主站首页
- `Gallery` 成为主站统一相册入口
- 留言相关入口统一保留在主站留言墙

这个调整看起来很小，但对整个站点结构很关键。

因为它意味着主站不再只是一个长页面，而是开始承担“整站导航中枢”的角色。

## 二、AI 固定检测脚本：不是刷量，而是给服务做轻量健康检查

今天还加入了一个 AI 固定检测脚本。

它的目标不是伪装真人访问，也不是刻意刷访问量，而是给 AI 助手做一个轻量的健康检查。

因为我的网站现在已经有了 AI 助手和 AI 使用统计页面。  
当 AI 接口变成站点的一部分后，我就需要知道它是否还能正常响应。

这次脚本的思路是：

| 设计点 | 作用 |
| --- | --- |
| 每天生成随机时间点 | 避免所有请求集中在同一秒触发 |
| 每次只问固定轻量问题 | 降低 token 与服务器压力 |
| 请求头明确标识 healthcheck | 不伪装真人访问 |
| 设置超时 | 避免异常请求长时间挂住 |
| 记录失败 | 方便后续判断服务是否不稳定 |
| 设置每日上限 | 避免无限循环或打爆接口 |

我现在更愿意把它理解为：

> 给 AI 助手加一个“每天自动问候一下”的状态探针。

它会计入当前 AI 统计，因为我希望统计页面真实反映这个服务被调用的情况。  
但从设计立场上，它不是为了制造虚假访问，而是为了确认服务仍然可用。

这也是个人站点走向“服务化”以后很自然的一步。

以前页面只要能打开就行。  
现在如果有 AI、登录、留言、统计、后端 API，就要开始关心：

- 服务有没有活着
- 请求有没有超时
- 失败有没有记录
- 自动任务会不会给服务器造成压力

我的服务器配置并不夸张，只有 2 核 2G，所以这种检测必须轻量。

这次没有选择复杂监控系统，而是选择一个更适合当前阶段的方式：  
**低频、固定、可记录、可控上限。**

## 三、相册迁移：把分散在 blog 和 build 的视觉内容收回主站

今天的另一个结构性调整，是相册迁移。

之前 `blog` 和 `build` 里都存在相册或类似视觉展示入口。  
这样虽然每个子站都能放图片，但久了之后会产生一个问题：

> 访客不知道相册到底应该去哪里看，维护时我也不知道图片入口应该归属哪一边。

所以这次把相册统一迁移到主站，作为导航栏里的 `Gallery`。

这个调整之后，分工更清楚：

| 模块 | 调整后的定位 |
| --- | --- |
| `main / Gallery` | 统一视觉档案和壁纸入口 |
| `blog` | 专注 Hexo + Butterfly 长文阅读 |
| `build` | 专注建站成长记录 |
| 主站留言墙 | 统一承载留言互动 |

这样做的好处是：

- 图片内容不再散落在多个子站
- 主站成为视觉资产的统一入口
- `blog` 可以更专注文章阅读
- `build` 可以更专注项目成长记录
- 后续图库、壁纸、抽奖礼物都能复用同一套相册数据

这次 Source Lottery 里第三种结果“壁纸礼物”，其实也正好受益于这个调整。

因为壁纸不再需要临时写死到某个单独卡片里，而是可以从主站 Gallery 的相册档案里随机抽取。  
这让相册不只是一个静态页面，也开始变成其他交互的内容来源。

## 四、Source Lottery：把普通来源卡片变成访客参与的老虎机

今天最重要的更新，是 `Source Lottery`。

原本页面里有一个“本站参考来源”的卡片。  
它的功能很直接：告诉访客本站参考了哪些项目、哪些部分来自学习、哪些部分是自己改造。

这个内容本身很重要，因为我一直希望项目保持清晰态度：

- 尊重原作者
- 说明参考来源
- 不把参考学习包装成完全原创
- 同时继续推进自己的原创改造

但是，原来的表达方式太普通了。  
它就是一个说明卡片，读者看完就结束。

而今天我想做的是：

> 既然首页正在原创化，那连“说明参考来源”这个位置，也应该变成更有本站气质的互动。

于是 `Source Lottery` 出现了。

## Source Lottery 的核心设定

`Source Lottery` 是一个小型游戏厅风格的老虎机。

它的基本逻辑是：

1. 访客触发抽奖
2. 机器随机生成三个数字
3. 数字范围对应三类结果
4. 结果不塞在老虎机内部，而是以独立卡片弹到屏幕中央

抽奖号码目前按区间映射：

| 号码区间 | 结果类型 | 展示方式 |
| --- | --- | --- |
| `000-332` | 主站首页参考来源说明 | 文字介绍卡片 |
| `333-665` | `blog` / Butterfly 来源说明 | 文字介绍卡片 |
| `666-999` | Gallery 壁纸礼物 | 纯壁纸镭射卡片 |

这里我特意做了一个边界：  
前两个结果只是文字说明，不出现壁纸；第三个结果才是纯壁纸礼物。

这样可以避免“来源说明”变成纯装饰，也能让壁纸礼物真正像一个隐藏奖励。

## 为什么用老虎机，而不是普通按钮

一开始这个位置可以有很多方案：

- 普通按钮
- 摇杆
- 抽卡
- 游戏机
- 弹窗说明
- 直接保留原卡片

最后选择老虎机，是因为它同时满足几个条件：

| 需求 | 为什么老虎机合适 |
| --- | --- |
| 保留轻松感 | 抽奖比说明书更像互动 |
| 保留来源说明 | 结果卡片仍然可以承载文字 |
| 增加记忆点 | 三位数字比普通按钮更容易让人记住 |
| 适合原创化 | 结构已经不是原来的普通参考卡片 |
| 能接入 Gallery | 壁纸奖励可以使用相册数据 |

我喜欢这个方向的一点在于：  
它不是为了炫技而加交互，而是把一个原本必须存在的内容，换成了更像本站的表达方式。

## Source Lottery 的交互结构

这次不是只做了一个机器外壳。  
为了让它真的像一个可玩的模块，还加了几个独立交互：

| 组件 | 作用 |
| --- | --- |
| 小型老虎机 | 抽出三位数字 |
| 桌面端摇杆 | 电脑端主要触发方式 |
| 手机端 START 按钮 | 更适合触屏操作 |
| 魔法书手册 | 解释抽奖规则 |
| 历史记录 | 查看之前抽到过的号码 |
| 外挂模式 | 指定号码，测试或主动查看某类结果 |
| 独立结果卡片 | 抽奖结果在屏幕中央弹出 |

这里有一个很重要的设计决策：

> 手册、历史记录、外挂模式和抽奖结果，都不要挤在老虎机内部。

如果所有东西都塞进一个框里，机器会变得又挤又乱。  
所以这次把它们拆成独立卡片：

- 魔法书在左侧白色区域
- 历史记录和外挂入口在右侧白色区域
- 点击后弹出独立卡片
- 抽奖结果也弹出在屏幕中央

这种处理让老虎机本身可以保持“小而集中”，而周围工具又不会丢失。

## 为什么结果要独立弹出

之前有一版尝试把结果塞在老虎机框里，但整体感觉不对。  
问题主要有三个：

1. 结果内容会挤压机器主体
2. 壁纸展示空间太小
3. 文字来源卡片和壁纸奖励无法形成足够区别

所以最后改成：

> 机器负责抽奖，结果负责独立登场。

这让交互逻辑更像真实抽奖：

- 你在机器上操作
- 机器给出号码
- 奖品从屏幕中央弹出来

尤其是第三种壁纸结果，独立弹出后可以做成立体镭射卡片。

壁纸本身是横屏的，如果强行塞进普通小框，就会裁掉太多内容。  
所以这次壁纸结果做了适配：

- 保留壁纸比例
- 使用适合图片的立体卡片容器
- 不把图片硬裁成方形
- 让壁纸像“抽中的收藏卡”一样展示

这样第三种结果才真正有“奖励”的感觉。

## 手机端为什么要单独调整

这次 Source Lottery 在手机端花了不少时间。

因为电脑端可以用左右空间放魔法书、历史记录和外挂入口，但手机端屏幕太窄，如果照搬桌面布局，就会出现遮挡和挤压。

手机端主要调整了这些点：

- 魔法书放在左侧
- 历史记录和外挂放在右侧
- 历史和外挂入口改成圆形按钮
- 后来又恢复为横向并列，更符合手机端观看
- 结果弹窗层级提高，避免被 `Leave a Message` 区域盖住
- 魔法书、历史记录、外挂弹窗都和结果弹窗一样保持独立覆盖
- 手机端老虎机体积压缩，避免占据过多高度

这一步很重要。

因为一个交互如果只在电脑端好看，不能算真正落地。  
尤其是这个网站本身会有很多手机端访问，移动端的遮挡、层级、空白和按钮位置都必须单独处理。

这次调完后，我更明确地感觉到：

> 首页原创化不只是桌面端视觉，而是整套体验都要能站住。

## 第三种结果的加载保护

后面我又补了一处很重要的小体验：  
如果抽到第三种结果，也就是 Gallery 高清壁纸礼物，电脑端和手机端都会先打开完整的镭射卡片容器。

在高清壁纸真正加载完成之前，卡片里不会空着，也不会只剩一个小点和 `Close` 按钮。  
它会先显示：

> 高清壁纸正在路上...

等图片加载完成后，再把壁纸淡入显示出来。

这个修复很小，但对观感很重要。  
因为壁纸是网络图片，加载速度会受到设备、网络、COS/CDN 状态影响。如果没有加载占位，访客很容易以为抽奖弹窗坏了。

现在的逻辑更稳：

| 状态 | 访客看到什么 |
| --- | --- |
| 抽到 `666-999` | 镭射卡片立即打开 |
| 图片加载中 | 显示“高清壁纸正在路上...” |
| 图片加载完成 | 高清壁纸在卡片中淡入 |
| 图片临时失败 | 保持加载提示，而不是显示空壳 |

第一次补占位后，我又继续加固了一次：抽到壁纸结果的那一刻就同步把卡片状态重置为 `loading`，从历史记录重新打开壁纸时也重置为 `loading`，图片地址变化时强制重新挂载图片元素。这样就不会沿用上一张图的 `loaded` 状态，也不会因为图片地址为空而出现一个空 `<img>`。

这也让我更确定一件事：  
一个原创交互不只要有概念，还要照顾异常状态。否则越有设计感的组件，坏掉时反而越容易让人误解。

## Source Lottery 对原创化的意义

这次我最想记录的，其实不是“我做了一个老虎机”，而是它在原创化里的意义。

原来的参考来源卡片，本质上还是一个常规网页组件：

- 一张卡片
- 一段说明
- 几个链接
- 一个声明

现在它变成了：

- 有抽奖行为
- 有三位数字记忆点
- 有魔法书手册
- 有历史记录
- 有外挂模式
- 有文字卡片和壁纸奖励两种结果
- 有桌面与手机不同的操作方式
- 有和 Gallery 数据联动的隐藏礼物

这已经不是单纯“换皮”。

因为它改变了内容被阅读的方式。  
访客不再只是被动看到来源说明，而是通过一次小小的抽奖去遇见这些信息。

这让我觉得它比单纯改颜色、换背景、换图标更接近原创化。

因为真正的原创感，往往来自：

- 交互逻辑
- 内容组织方式
- 访客参与方式
- 页面记忆点
- 功能之间的联动

`Source Lottery` 同时碰到了这些点。

## 今天更新后的结构关系

这轮改完后，我觉得主站结构比之前更清楚了。

| 模块 | 今天之后的作用 |
| --- | --- |
| `HOME` | 所有页面回到主站的统一入口 |
| `Gallery` | 主站统一相册和壁纸档案 |
| `Guestbook` | 主站唯一留言墙 |
| `AI Health Check` | 轻量确认 AI 助手服务是否可用 |
| `Source Lottery` | 用游戏化方式说明参考来源、博客来源和壁纸礼物 |
| `blog` | 专注正式文章与 Butterfly 阅读体验 |
| `build` | 继续记录网站成长与原创化过程 |

这是一种很舒服的分工：

- 主站负责视觉、入口和互动
- 博客负责阅读
- 成长博客负责记录
- 后端负责服务
- AI 检测负责确认服务状态
- Gallery 负责视觉资产沉淀
- Source Lottery 把“来源说明”变成可玩的原创表达

## 我对这次更新的判断

如果只看代码层面，今天做的是几个分散功能：

- 改导航
- 加检测脚本
- 迁移相册
- 改 Source Lottery
- 调移动端层级
- 修控制台里的 `fetchPriority` 警告

但从网站成长角度看，它们其实指向同一个方向：

> 这个网站正在从“参考优秀作品做出来的首页”，继续长成“拥有自己结构和互动语言的个人站点”。

我不想假装它已经完全脱离参考来源。  
但我也能很清楚地看到，它已经不是简单照着一个项目复制出来的页面。

它现在有了：

- 自己的导航逻辑
- 自己的相册入口
- 自己的 AI 服务检测方式
- 自己的留言入口
- 自己的来源说明方式
- 自己的游戏化抽奖模块
- 自己的壁纸奖励逻辑

这就是原创化继续推进的证据。

## 下一步还可以怎么继续

接下来我还可以继续往几个方向推进：

| 方向 | 目标 |
| --- | --- |
| 完善 Source Lottery 规则 | 让号码、奖励和相册之间有更丰富的对应关系 |
| 增加 Gallery 分类 | 让壁纸奖励来源更清晰 |
| 优化 AI 检测记录展示 | 把健康检查与 AI 统计区分得更可读 |
| 继续整理导航 | 让主站、blog、build 的入口关系更统一 |
| 继续原创化剩余模块 | 把仍然像参考项目的部分逐步换成自己的表达 |
| 写更多成长记录 | 把每一次重要更新都沉淀下来 |

我很喜欢今天这次更新的一点是：  
它没有只停留在“页面更好看了”，而是让网站的几个模块之间开始产生联系。

相册不只是相册，它可以成为抽奖奖励。  
来源说明不只是说明，它可以成为游戏结果。  
AI 统计不只是数字，它可以通过健康检查保持持续观察。  
导航不只是菜单，它开始承担整站入口秩序。

这就是一个个人网站慢慢长大的感觉。

上一篇：

- [建站成长记录 04｜首页原创化改造：从参考学习到个人表达](https://taozhiyy.top/build/post/site-growth-04-homepage-originalization)

---en---

# Site Growth Log 05: Originalization Continues Through Structure, Not Just Visuals

The previous growth log recorded a major round of homepage originalization: the Hero compass, the About display, the archive-book idea in Features, the paper-note Story section, the learning statement in the footer, and mobile adaptation.

Today's round pushes that direction further.  
It is not only about making the page prettier. It is about organizing the site's entrances, content boundaries, and visitor interactions more clearly.

This update mainly includes four parts:

- navigation bar fixes
- an AI health-check scheduler
- gallery migration into the main site
- transforming the source-reference card into `Source Lottery`, a small arcade-style slot interaction

The most important part is the last one.  
It is not simply replacing one card with another card. It changes a static source statement into a small interaction that visitors can participate in.

That is how I now understand originalization:

> Originalization is not about erasing every trace of a reference project overnight. It is about repeatedly asking every part of the page: can this feel more like my own website?

## Why this round matters

If the previous round focused on whether the homepage visually felt like me, this round focuses on another question:

> Do the site's entrances, content, and explanations also have their own organization logic?

A personal website cannot stand only on beautiful animations.  
Visitors also need to know:

- how to return home
- where to read articles
- where to view the gallery
- where to leave a message
- whether the AI assistant is stable
- how the site explains references and original modifications

These details are less flashy than Hero animation, but they decide whether the site can become a maintainable whole.

So today's update is not just more decoration. It is a structural cleanup.

## 1. Navigation: making HOME the real return point

The first adjustment was the navigation bar.

The homepage previously had anchor-style items such as `about`, `contact`, and `end`.  
That pattern is common for single-page showcase sites, but my site is no longer only a single showcase page.

The main site now has several real entrances:

| Entrance | Current role |
| --- | --- |
| `/` | Main homepage and the first entrance of the whole site |
| `/gallery` | Unified gallery entrance |
| `/guestbook` | Main guestbook |
| `/ai-traffic` | AI usage statistics |
| `/bili` | Bili-related page |
| `/blog/` | Hexo + Butterfly blog |
| `/build/` | Growth blog |

In this structure, the navigation bar does not need to keep splitting the homepage into several anchors.  
What it needs more is a stable way for visitors to return to the main site from anywhere.

So the previous `about`, `contact`, and `end` items were replaced with `HOME`.

The current logic is clearer:

- on the homepage, `HOME` returns to the Hero section
- on subpages, `HOME` returns to the main homepage
- `Gallery` becomes the unified visual archive entrance
- guestbook interaction is kept in the main guestbook

This looks like a small change, but it matters for the whole site structure.

It means the main site is no longer just a long page. It is becoming the navigation hub of the entire website.

## 2. AI health checks: lightweight service observation, not fake traffic

Another update today was the AI health-check scheduler.

Its purpose is not to impersonate real users, and it is not designed as fake traffic.  
It is a lightweight way to confirm that the AI assistant can still respond.

The site now has an AI assistant and an AI usage statistics page.  
Once the AI endpoint becomes part of the site, I need to know whether it remains available.

The script is designed around these choices:

| Design point | Purpose |
| --- | --- |
| generate random daily time points | avoid all requests happening at the same moment |
| ask one fixed lightweight prompt | reduce token usage and server pressure |
| identify itself as a health check | avoid pretending to be a human visitor |
| set request timeouts | prevent hanging requests |
| record failures | make instability visible later |
| keep a daily limit | avoid loops or excessive requests |

I now see it as:

> a small daily status probe that says hello to the AI assistant.

It is counted in the current AI statistics because I want the statistics page to reflect actual calls to the service.  
But the design intention is still service observation, not artificial traffic.

This is a natural step once a personal website starts becoming service-based.

Before, a page only needed to open.  
Now, with AI, login, guestbook, stats, and backend APIs, I need to care about:

- whether the service is alive
- whether requests time out
- whether failures are recorded
- whether scheduled tasks create pressure

My server is modest, only 2 cores and 2 GB RAM, so the check has to stay lightweight.

Instead of a complex monitoring system, this stage uses a simpler approach:  
**low frequency, fixed behavior, failure records, and a controlled daily limit.**

## 3. Gallery migration: bringing scattered visual content back to the main site

Another structural adjustment was gallery migration.

Before this update, both `blog` and `build` had gallery-like visual entrances.  
That works temporarily, but over time it creates a problem:

> visitors do not know where the real gallery is, and I do not know which module should own visual assets.

So the gallery was unified into the main site as `Gallery`.

After the adjustment, the responsibilities are clearer:

| Module | New role |
| --- | --- |
| `main / Gallery` | unified visual archive and wallpaper entrance |
| `blog` | long-form reading with Hexo + Butterfly |
| `build` | site growth records |
| main guestbook | unified message interaction |

The benefits are:

- images no longer scatter across multiple subsites
- the main site becomes the unified visual asset entrance
- `blog` can focus on article reading
- `build` can focus on project growth records
- future gallery, wallpaper, and lottery gift features can reuse the same album data

The third result in Source Lottery, the wallpaper gift, benefits directly from this decision.

Wallpaper no longer needs to be hard-coded inside a separate card. It can be randomly drawn from the main Gallery archive.  
That makes the gallery more than a static page. It becomes a content source for other interactions.

## 4. Source Lottery: turning a source card into an arcade interaction

The most important update today is `Source Lottery`.

Originally, this section was a source-reference card.  
Its job was straightforward: explain what projects the site learned from, which parts came from references, and which parts were personally modified.

The content itself is important, because I want the project to keep a clear attitude:

- respect original authors
- state references clearly
- do not present reference learning as complete originality
- continue pushing original modifications forward

But the old expression was too ordinary.  
It was a card, a few lines of explanation, and then the reader moved on.

What I wanted today was:

> if the homepage is becoming original, even the source statement should become an interaction that belongs to this site.

That is how `Source Lottery` appeared.

## The core idea of Source Lottery

`Source Lottery` is a small arcade-style slot machine.

Its basic flow is:

1. the visitor triggers a draw
2. the machine generates three digits
3. the number range maps to three result types
4. the result appears as an independent card in the center of the screen

The current number mapping is:

| Number range | Result type | Display |
| --- | --- | --- |
| `000-332` | homepage reference source statement | text card |
| `333-665` | `blog` / Butterfly source statement | text card |
| `666-999` | Gallery wallpaper gift | wallpaper holographic card |

I deliberately kept a boundary here:  
the first two results are text-only source cards, and only the third result is a pure wallpaper gift.

That prevents the source explanation from becoming mere decoration, while still making the wallpaper feel like a hidden reward.

## Why a slot machine instead of a normal button

There were many possible directions for this section:

- a normal button
- a lever
- a card draw
- an arcade machine
- a modal explanation
- keeping the original card

The final choice was a slot machine because it satisfies several goals at once:

| Need | Why the slot machine fits |
| --- | --- |
| keep it playful | a draw feels more interactive than a manual |
| preserve source explanation | result cards can still carry text |
| create a memory point | three digits are more memorable than a plain button |
| support originalization | the structure is no longer the old source card |
| connect to Gallery | wallpaper rewards can reuse gallery data |

What I like about this direction is that the interaction is not added only for show.  
It changes a necessary piece of content into a more site-specific expression.

## Interaction structure

This update did not only create a machine shell.  
To make the module feel genuinely playable, it also includes several supporting interactions:

| Component | Role |
| --- | --- |
| small slot machine | draws three digits |
| desktop lever | main desktop trigger |
| mobile START button | better for touch screens |
| magic-book manual | explains the rules |
| history panel | shows previous draw numbers |
| cheat mode | allows a specified number |
| independent result card | displays the result in the center |

One key decision was:

> the manual, history, cheat mode, and draw result should not be squeezed inside the slot machine.

If everything is stuffed into one box, the machine becomes crowded and messy.  
So they were separated into independent cards:

- the magic book sits in the left white area
- history and cheat controls sit in the right white area
- clicking them opens independent cards
- draw results also appear in the center of the screen

This lets the machine stay small and focused while still keeping the surrounding tools available.

## Why results pop out independently

One earlier version tried to put the result inside the machine frame, but it felt wrong.  
There were three main problems:

1. the result content squeezed the machine body
2. wallpaper had too little display space
3. text source cards and wallpaper rewards did not feel different enough

So the final logic became:

> the machine performs the draw, and the prize appears separately.

This feels closer to a real lottery:

- you operate the machine
- the machine gives you a number
- the prize pops out in the center

This is especially important for the third wallpaper result, which can now become a 3D holographic card.

The wallpapers are landscape images. If they are forced into a small square frame, too much content gets cropped.  
So the wallpaper result now adapts better:

- it preserves the wallpaper ratio
- it uses a card container that fits the image
- it does not force the image into a square crop
- it feels like a collectible card drawn from the machine

That is what makes the third result feel like a real reward.

## Why mobile needed separate tuning

Source Lottery required quite a lot of mobile work.

Desktop has enough left and right space for the magic book, history, and cheat controls.  
Mobile screens are much narrower, so copying the desktop layout directly caused overlap and cramped spacing.

The mobile adjustments included:

- keeping the magic book on the left
- placing history and cheat controls on the right
- turning history and cheat into compact round controls
- later restoring them into a horizontal pair because it felt better on mobile
- raising modal layers so they are not covered by the `Leave a Message` section
- making manual, history, cheat, and result modals share the same independent overlay behavior
- compressing the mobile slot machine so it does not take too much height

This step matters.

An interaction that only works on desktop is not truly finished.  
Since this site will also be visited on phones, mobile layering, empty space, button placement, and modal behavior all need their own treatment.

After this round, I can say more clearly:

> homepage originalization is not only desktop visuals; the whole experience has to hold together.

## Loading protection for the third result

I later added one more important detail:  
when the visitor draws the third result, the Gallery wallpaper gift, both desktop and mobile now open the full holographic card immediately.

Before the high-resolution wallpaper finishes loading, the card no longer appears empty, and it no longer looks like only a tiny dot plus a `Close` button.  
It first shows:

> 高清壁纸正在路上...

Once the image finishes loading, the wallpaper fades into the card.

This is a small fix, but it matters a lot for perceived quality.  
Wallpapers are network images, so loading can be affected by the device, connection, and COS/CDN state. Without a loading placeholder, visitors may easily think the lottery modal is broken.

The logic is now more stable:

| State | What visitors see |
| --- | --- |
| drawing `666-999` | the holographic card opens immediately |
| image loading | “高清壁纸正在路上...” is shown |
| image loaded | the wallpaper fades into the card |
| temporary image failure | the loading message remains instead of an empty shell |

After the first placeholder attempt, I strengthened the logic again: the card is reset to `loading` at the exact moment a wallpaper result is drawn, history replay of a wallpaper result also resets to `loading`, and every new image URL remounts the image element. This prevents the modal from reusing a previous `loaded` state, and it also avoids rendering an empty `<img>` when the image URL is missing.

This also makes one thing clearer to me:  
an original interaction needs not only a concept, but also careful handling of edge states. Otherwise, the more designed a component is, the more confusing it becomes when something fails.

## What Source Lottery means for originalization

The most important thing I want to record is not simply “I made a slot machine.”  
It is what this module means for originalization.

The old source-reference card was still a conventional web component:

- one card
- some text
- a few links
- a statement

Now it has become:

- a draw action
- a three-digit memory point
- a magic-book manual
- history records
- cheat mode
- text cards and wallpaper rewards
- different desktop and mobile controls
- a hidden reward connected to Gallery data

That is not just reskinning.

It changes how the content is encountered.  
Visitors no longer only passively read the source statement. They meet it through a small draw.

That feels much closer to originalization than simply changing colors, backgrounds, or icons.

Because true originality often comes from:

- interaction logic
- content organization
- visitor participation
- memorable page moments
- relationships between features

`Source Lottery` touches all of those.

## The structure after today's update

After this round, the main site feels clearer.

| Module | Role after today's update |
| --- | --- |
| `HOME` | unified return entrance to the main site |
| `Gallery` | unified gallery and wallpaper archive |
| `Guestbook` | main message wall |
| `AI Health Check` | lightweight confirmation that the AI assistant is available |
| `Source Lottery` | game-like source explanation and wallpaper reward |
| `blog` | formal writing and Butterfly reading experience |
| `build` | growth records and originalization logs |

This division feels comfortable:

- the main site handles visual experience, entrances, and interaction
- the blog handles reading
- the growth blog handles documentation
- the backend handles services
- AI health checks keep the service observable
- Gallery stores visual assets
- Source Lottery turns source explanation into playable personal expression

## My judgment of this update

At the code level, today's work looks like several separate tasks:

- fixing navigation
- adding a health-check scheduler
- migrating gallery entrances
- building Source Lottery
- tuning mobile modal layers
- fixing the `fetchPriority` console warning

But from the site's growth perspective, they point in the same direction:

> this website is continuing to grow from a homepage inspired by a great reference project into a personal site with its own structure and interaction language.

I do not want to pretend it has completely left every reference behind.  
But I can also clearly see that it is no longer just a copied page.

It now has:

- its own navigation logic
- its own gallery entrance
- its own AI service observation method
- its own guestbook entrance
- its own way to explain sources
- its own game-like lottery module
- its own wallpaper reward logic

That is evidence that originalization is still moving forward.

## What can come next

There are several directions I can continue:

| Direction | Goal |
| --- | --- |
| improve Source Lottery rules | make number ranges, rewards, and gallery content richer |
| expand Gallery categories | make wallpaper reward sources clearer |
| improve AI check display | make health checks more readable inside AI stats |
| keep organizing navigation | make main, blog, and build entrances more unified |
| originalize remaining modules | gradually replace remaining reference-like structures |
| write more growth logs | preserve every major update as part of the site's history |

What I like about today's update is that it does not stop at “the page looks better.”  
It makes different modules of the site start talking to each other.

Gallery is not only a gallery. It can become a lottery reward source.  
The source statement is not only a statement. It can become a game result.  
AI statistics are not only numbers. They can be supported by lightweight health checks.  
Navigation is not only a menu. It starts to define the order of the whole site.

That is how a personal website slowly grows up.

Previous in the series:

- [Site Growth Log 04 | Homepage Originalization: From Reference Learning to Personal Expression](https://taozhiyy.top/build/post/site-growth-04-homepage-originalization)
