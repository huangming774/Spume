---
date: 2026-06-08
slug: site-growth-07-moments-controller-scroll-release
title_zh: 建站成长记录 07｜碎语页面、站长发布链路与滚动修复
title_en: Site Growth Log 07 | Moments Page, Owner Publishing Flow, and Scroll Fixes
excerpt_zh: 这篇记录 6 月 8 日的一组收尾更新：碎语页面正式迁到主站导航并接入站长发布接口，友链数据修正为 Leyili 花园，首页与其他页面的双层滚动问题完成修复，同时 README 和后续计划也同步补齐。
excerpt_en: This post records a June 8 polish pass: the Moments page now lives in the main navigation and can be published from the owner console, the Leyili Garden friend link was corrected, nested page scrolling was fixed, and the README/future-plan notes were brought up to date.
words: 3600
reads: 28
minutes: 13
---

# 建站成长记录 07：碎语页面迁进主站之后，发布链路也补齐了

这几次更新更像一次“把散落的边角收回来”的整理。

前面已经把站长控制台、友链发布、相册发布、COS 上传和后端安全边界打通了。这一次的重点，是把更细的内容体验和文档状态补齐：

- “说说”正式改名为“碎语”，并作为主站顶部导航入口进入 `/moments`
- 碎语页面使用多种全息卡片模块，而不是单一列表
- 站长控制台新增真实碎语发布流程，后端写入 `main/src/data/moments.js`
- 友链里 `https://930309.xyz/` 的信息修正为 `Leyili 花园`
- 首页双层滚动条问题修复，并检查其他页面不再出现页面级第二滚动容器
- README、后端控制器说明和 build 记录同步更新

它不是一个特别大的功能发布，但它让网站的日常维护更顺手了。

## 碎语页面为什么从“说说”变成“碎语”

一开始这个入口叫“说说”，更像传统博客里的短动态。

后来我觉得“碎语”更适合这个站：

- 它不强调社交平台式更新
- 更像个人站里偶然留下的一句话
- 和主站偏梦幻、轻叙事的视觉风格更贴
- 可以容纳心事、实验记录、吐槽和短诗一样的内容

所以这次把页面标题、导航入口和测试约束统一成“碎语”。

现在主站顶部导航直接进入 `/moments`，不再把它塞在 Features 的某个卡片里。这样它就和博客、成长记录、画廊、友链一样，成为站内的正式内容页。

## 碎语页面现在是什么结构

前端这边，碎语页面由三个部分组成：

| 部分 | 文件 | 作用 |
| --- | --- | --- |
| 路由 | `main/src/App.jsx` | 挂载 `/moments` 页面 |
| 页面 | `main/src/pages/MomentsPage.jsx` | 渲染标题、背景光效和碎语列表 |
| 数据 | `main/src/data/moments.js` | 保存年份、日期、分类、视觉语气、模块和正文行 |

每一条碎语不是简单的纯文本。

它会带上：

- `year`
- `date`
- `type`
- `tone`
- `module`
- `lines`

其中 `tone` 和 `module` 决定卡片的颜色和样式模块，例如 postcard、ticket、watercolor、poem、journal、ribbon。这样短内容虽然轻，但页面不会变成一排重复的小框。

这也是我现在越来越喜欢的方向：内容可以短，但承载它的界面不能敷衍。

## 站长控制台也可以发布碎语

只做一个静态碎语页其实还不够。

如果每次想写一句短内容都要手动打开数据文件、改数组、提交、推送，那这个入口很快就会闲置。

所以这次后端也补上了碎语发布控制器：

| 控制器 | 路由 | 写入目标 |
| --- | --- | --- |
| `owner_moment_publish.go` | `POST /api/owner/moments` | `main/src/data/moments.js` |

请求体大致是：

```json
{
  "year": "2026",
  "date": "6.8",
  "type": "心事",
  "content": "一滴泪真正的重量取决于它落在谁的心上"
}
```

后端会做几件事：

1. 确认当前 session 是站长，并且已经通过二次验证
2. 读取 GitHub 仓库里的 `main/src/data/moments.js`
3. 校验年份、日期、分类和正文
4. 按已有数据数量分配下一组 `tone` 和 `module`
5. 把新碎语按日期插入到更靠前的位置
6. 通过 GitHub Contents API 提交真实 commit
7. 返回路径、分支、commit SHA 和仓库链接

这样一来，碎语和友链、文章、相册一样，都进入了同一套 owner-only 发布链路。

前端负责表单和预览，后端负责鉴权、校验和写仓库。

## 友链信息修正

这次还修正了一条友链数据。

`https://930309.xyz/` 对应的信息应该是：

| 字段 | 内容 |
| --- | --- |
| 名称 | Leyili 花园 |
| 图标 | `https://photo.930309.xyz/lcj.svg` |
| 链接 | `https://930309.xyz/` |
| 描述 | 小小后花园~~~ |

之前有一部分内容填错，容易和另一个友链混在一起。

现在 `090909.top` 仍然保留为“他说”，`930309.xyz` 则恢复为 `Leyili 花园`。这类数据看起来小，但友链页最重要的就是准确尊重对方站点，所以这里单独补了一条测试保护。

## 双层滚动条问题

还有一个视觉体验问题也顺手处理了：部分页面右侧会出现两条竖向滚动条。

这通常意味着页面外层和内部某个容器都在抢纵向滚动。

这次修复集中在主站布局和 Hero 相关容器上：

- `Hero.jsx` 从横向隐藏改成整体裁切，避免 Hero 内部制造额外滚动
- `SiteLayout.jsx` 使用更克制的横向裁切，减少外层布局和页面内容的滚动冲突
- 新增布局约束测试，防止后续再把双滚动问题带回来
- 扫描主站关键路由，确认首页和其他页面没有页面级第二滚动容器

这个问题很容易被忽略。

页面能滚不代表滚动体验正确。双层滚动条会让触控板、鼠标滚轮和移动端手势都变得奇怪，所以它应该作为布局质量问题处理，而不是只当成小瑕疵。

## 文档这次补了什么

这次 README 同步补了三类信息。

第一类是主站体验：

- `/moments` 碎语页面
- 顶部导航入口
- 碎语数据文件
- 卡片视觉模块

第二类是后端控制器：

- `POST /api/owner/moments`
- `owner_moment_publish.go`
- 发布请求形状
- 目标数据文件和 GitHub commit 结果

第三类是后续计划：

- 服务器数据和状态监测
- 打造生成图展示页面
- 设计艺术字主页

我希望 README 不只是“项目能跑起来”的说明，也能记录当前网站真实发展到了哪一步。

## 接下来的计划

接下来比较明确的三个方向是：

| 计划 | 想解决的问题 |
| --- | --- |
| 服务器数据和状态监测 | 在站长控制台里看到服务运行状态、资源使用、同步状态和关键数据趋势 |
| 打造生成图展示页面 | 把已经生成并保存到 COS 的图片整理成可浏览、可回看、可管理的展示页 |
| 设计艺术字主页 | 继续探索更有辨识度的首页标题、艺术字和主视觉表达 |

第一个方向偏后台基础设施，第二个方向偏内容资产，第三个方向偏视觉识别。

它们不像碎语页面那样都能一眼看出功能边界，但会让网站从“能发布内容”继续往“能长期维护、运营和表达自己”靠近。

现在这个阶段，我更在意网站能不能被自己持续使用。

如果一个功能只适合演示，不适合日常维护，那它最终会变成摆设。碎语发布、友链发布、图片上传和后续监测能力，本质上都是在减少日常维护阻力。

网站慢慢长成一个真正可用的小系统，这件事本身就很值得记录。

---en---

# Site Growth Log 07: After the Moments Page Moved Into the Main Site, the Publishing Flow Caught Up

This update is a cleanup and documentation pass around several recent changes.

The main points are:

- the old “说说” entry is now named “碎语” and lives at `/moments`
- the Moments page is available from the main top navigation
- Moments are displayed as varied holographic cards instead of a plain list
- the owner console can publish a real Moment through the backend
- the backend writes into `main/src/data/moments.js` through GitHub
- the `https://930309.xyz/` friend link was corrected to `Leyili 花园`
- the main site double-scroll issue was fixed and checked across pages
- README, backend controller notes, and the build log were updated

It is not a huge feature release, but it makes the site easier to maintain day to day.

## Why “Moments” became 碎语

The page started closer to a traditional short-status page.

“碎语” fits this site better:

- it feels less like a social feed
- it can hold short thoughts, experiments, notes, and poetic fragments
- it matches the softer personal-site tone
- it works as a first-class content page rather than a hidden card entry

The page now has a direct top-nav entry and is mounted at `/moments`.

## Page structure

The frontend pieces are:

| Part | File | Role |
| --- | --- | --- |
| Route | `main/src/App.jsx` | Mounts `/moments` |
| Page | `main/src/pages/MomentsPage.jsx` | Renders the hero, background glows, and moment cards |
| Data | `main/src/data/moments.js` | Stores year, date, type, tone, module, and text lines |

Each Moment carries its visual tone and module, such as postcard, ticket, watercolor, poem, journal, or ribbon. This keeps the page varied while still staying inside one visual family.

## Owner publishing flow

The new backend route is:

| Controller | Route | Target |
| --- | --- | --- |
| `owner_moment_publish.go` | `POST /api/owner/moments` | `main/src/data/moments.js` |

The request shape is roughly:

```json
{
  "year": "2026",
  "date": "6.8",
  "type": "心事",
  "content": "一滴泪真正的重量取决于它落在谁的心上"
}
```

The backend:

1. verifies the owner session
2. reads the current Moments data from GitHub
3. validates year, date, type, and content
4. assigns the next visual tone/module
5. inserts the new item in date order
6. commits the update through the GitHub Contents API
7. returns path, branch, commit SHA, and repository URL

This puts Moments into the same owner-only publishing model as articles, friend links, and gallery images.

## Friend link correction

The `https://930309.xyz/` friend link should be:

| Field | Value |
| --- | --- |
| Name | Leyili 花园 |
| Icon | `https://photo.930309.xyz/lcj.svg` |
| URL | `https://930309.xyz/` |
| Description | 小小后花园~~~ |

`090909.top` remains the separate “他说” entry. The data and tests now protect that distinction.

## Double-scroll fix

Some pages previously showed two vertical scrollbars. That usually means both the outer page and an inner container are competing for vertical scrolling.

The fix tightened the main layout and Hero overflow behavior, then added tests to guard the expected layout classes. The route scan confirmed that the homepage and other main pages no longer expose a page-level second scroll container.

Small layout bugs like this matter because scroll behavior is part of how the site feels. If scrolling is awkward, the page feels less finished even when the visuals are good.

## Documentation and next steps

The README updates now cover:

- the `/moments` page
- the owner Moment publishing controller
- the relevant request shape and data target
- the latest friend-link and scroll fixes
- the next backend plans
- the artistic lettering homepage direction

The next planned directions are:

| Plan | Goal |
| --- | --- |
| Server data and status monitoring | Surface service health, resource usage, sync state, and key data trends in the owner console |
| Generated-image showcase page | Turn generated COS images into a browsable, reusable, and manageable site gallery |
| Artistic lettering homepage | Explore a more recognizable homepage title, lettering system, and main visual identity |

Some of these are less visual than the Moments page, but they matter for long-term maintenance, content reuse, and a stronger site identity.

The site is slowly moving from “a set of pages” toward a small system I can actually run, update, and take care of over time.
