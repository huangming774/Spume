---
title: 新博客搭建记——用 Vibe Coding 打造「桃之夭夭の创作屋」
date: 2026-05-17 22:00:00
updated: 2026-05-18
sticky: 999999
tags: [VibeCoding, Claude Code, Codex, Cursor, React, Vite, Vercel, MCP, 个人主页]
categories: [项目]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/homepage-hero.png
description: 用一天时间搭建全新个人主页，再用一天从静态 HTML 迁移到 React SPA 架构，接入 MCP 组件生态。22 篇文章从 Hexo 博客完整迁移，全程 Vibe Coding。
---

{% note primary %}
全新个人主页——{% span blue, 桃之夭夭の创作屋 %}，从零到上线再到架构升级，全程用 **Vibe Coding** 完成。

地址：{% span blue, https://tzyy11.vercel.app %}

本文记录了这个站的两次重大迭代：第一阶段静态 HTML 搭建，第二阶段 React SPA 架构迁移。
{% endnote %}

---

## 为什么要建新站

在这之前，我已经有两个博客了：

1. **{% span blue, bistutzyy.github.io %}** — Hexo + Butterfly 主题，写了近两年
2. **{% span purple, blog1-reimu.vercel.app %}** — Hexo + Reimu 主题，年初刚搭的

两个博客各有特色，但问题也很明显：{% span red, 内容分散、管理碎片化、主题定制受限 %}。每次想改个样式都要折腾半天配置，Butterfly 标签和 Reimu 标签还不互通，迁移文章特别痛苦。

{% note info %}
核心诉求其实很简单：**有一个完全由自己掌控的空间**——不为搜索引擎优化，不为流量变现，纯粹用来记录和展示。
{% endnote %}

于是我决定用 AI 工具从零搭建第三个站——一个完全自由的个人创作屋。

---

## 第一阶段：三款 AI 工具搭建静态 HTML

整个搭建过程像一个接力赛，三款 AI 工具各司其职：

{% folding blue, OpenAI Codex — 主页整体设计 %}
Codex 负责了主页的视觉框架。需求是"设计一个带侧边栏的个人主页，日系插画风格，清新可爱"。

它生成了完整的 `index.html`：

- 左侧导航栏（首页/归档/文章/说说/相册/友链/留言墙）
- 中间大标题 "Hello" 渐变色 + 手写体
- 双语标语 "有趣的灵魂终会遇见"
- 右侧动漫少女角色立绘
- 三只漂浮小猫插图
- 版权信息 + 社交图标

Codex 对视觉风格的理解很到位——柔和的浅色调、圆角设计、合适的留白。
{% endfolding %}

{% folding green, Cursor — 碎片化页面设计 %}
Cursor 负责了独立页面：

- **相册页**：卡片式布局，点击进入子页面查看照片网格
- **友链页**：本站信息 + 友链卡片，带说明和申请指南
- **留言墙**：渐变色标题、表单三栏布局、留言卡片带糖果色头像
- **归档页**：年份时间线，文章按日期排列

Cursor 的优势在于碎片化任务——给它具体需求，它能快速生成符合整体风格的代码。
{% endfolding %}

{% folding purple, Claude Code + DeepSeek V4 Pro — 文章体系与全局调优 %}
Claude Code 负责了最核心的内容系统：

1. **build_content.py 构建脚本**：从 Markdown 文章 + frontmatter 生成文章列表、详情页、说说页、归档时间线
2. **21 篇文章迁移**：从 Hexo 博客的 Butterfly 标签语法全部转为标准 Markdown
3. **CSS 全局调优**：macOS 终端风格代码块、艺术字语录、表格样式、移动端适配、侧边栏折叠
4. **双语支持**：每篇文章中英双语，页面级语言切换

整个过程中，我在微信上给 Claude Code 发需求，它分析→生成→我审查修改，全程没有打开传统 IDE。
{% endfolding %}

---

## 第二阶段：从静态 HTML 到 React SPA

{% note success %}
第一阶段上线后发现静态 HTML 的扩展瓶颈——每新增一个页面需要手动修改多处导航链接，样式分散在 6 个文件中难以维护，无法接入现代组件生态。

于是用同样 Vibe Coding 的方式，花了一天时间，把整个站点从纯 HTML 迁移到了 React SPA。
{% endnote %}

### 迁移对比

| 维度 | 迁移前 | 迁移后 |
|------|--------|--------|
| 框架 | 纯 HTML/CSS/JS | {% span blue, Vite %} + {% span blue, React 19 %} + {% span blue, TypeScript %} |
| 路由 | 7 个独立 .html 文件 | react-router-dom v7 统一管理 |
| 样式 | 分散在 6 个 CSS 文件 | 统一 Vite CSS import 管线 |
| 组件 | 无，HTML 复制粘贴 | Layout / Sidebar / HelloHero 等组件化 |
| 构建 | Python 脚本生成 HTML | `npm run build` → Vite 打包到 dist/ |
| 部署 | 手动上传 | Vercel 自动构建部署 |
| 组件生态 | 无 | React Bits MCP 接入，520+ 动画组件可用 |

{% note info %}
迁移的核心原则：**视觉上 100% 保持原样**。所有原始 CSS（site-pages.css、site-weather.css、site-mobile-nav.css）一字不改地被引入，类名和 DOM 结构与原版一致。渐变背景、天气效果、移动端适配全部保留。
{% endnote %}

### 项目结构

```
homepage-project/
├── src/
│   ├── main.tsx              # React 入口
│   ├── App.tsx               # 路由配置（10 条路由）
│   ├── index.css             # 全局样式 & CSS import 管线
│   ├── components/
│   │   ├── Layout.tsx        # 全局布局（背景层 + 侧边栏 + 装饰元素）
│   │   ├── Sidebar.tsx       # 侧边栏导航
│   │   ├── HelloHero.tsx     # 首页 Hello 描边字体动画
│   │   └── Bilingual.tsx     # 中英双语切换组件
│   ├── pages/
│   │   ├── HomePage.tsx      # 首页
│   │   ├── ArticlesPage.tsx  # 文章列表
│   │   ├── PostPage.tsx      # 文章详情
│   │   ├── MomentsPage.tsx   # 说说
│   │   ├── ArchivesPage.tsx  # 归档
│   │   ├── GalleryPage.tsx   # 相册列表
│   │   ├── GalleryAlbumPage.tsx  # 相册详情
│   │   ├── FriendsPage.tsx   # 友链申请
│   │   ├── MessagePage.tsx   # 留言墙
│   │   └── BangumiPage.tsx   # B 站追番
│   ├── i18n/                 # 国际化 Context
│   ├── weather/              # 雨天效果 Context + Canvas
│   ├── data/                 # content.json 数据层
│   ├── styles/               # 页面级 CSS
│   └── lib/                  # 工具函数
├── content/
│   ├── articles/             # 22 篇 Markdown 文章
│   └── moments/              # 5 条说说
├── public/                   # 静态资源（图片等）
├── build_content.py          # Markdown → JSON 构建脚本
├── vercel.json               # Vercel 部署配置
└── vite.config.ts            # Vite 构建配置
```

### MCP：一句话装组件

迁移到 React 后，接入了 shadcn MCP 服务器，连接了 React Bits 组件仓库：

| MCP 操作 | 效果 |
|------|------|
| 列出组件 | 浏览 {% span blue, 520 个 %} 动画组件 |
| 搜索组件 | 按类别筛选（背景、文字动效、交互组件） |
| 查看详情 | 查看组件描述、依赖、文件数 |
| 获取命令 | 拿到 `npx shadcn add` 安装命令 |

每个组件提供 JS/TS × CSS/Tailwind 四种变体。以后给创作屋加新动效，直接对话「加一个极光背景」，AI 自动搜组件 → 安装依赖 → 写入代码。

---

## 功能设计

### 首页

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/homepage-hero.png" alt="首页效果" style="width:100%">

首页是整个创作屋的门面——动漫少女插画 + 手绘小猫 + 渐变色标题 + Hello SVG 描边动画，营造出"有趣的灵魂终会遇见"的氛围。

### 文章系统

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/homepage-articles.png" alt="文章界面" style="width:100%">

文章列表用卡片式布局，支持中英双语切换。代码块用了 macOS 终端风格——暗色背景 + 顶部标题栏 + 三色圆点 + highlight.js 语法高亮。

### 说说 — 随机艺术效果

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/homepage-shuoshuo.png" alt="说说界面" style="width:100%">

重点——说说页面里每条卡片的样式是 {% span red, 随机分配 %} 的：

{% folding orange, 说说随机样式一览 %}
- 极光渐变背景 + 描边
- 樱花粉色调
- 浅蓝渐变 + 水波纹理
- 车票风格（虚线边框 + TICKET 水印）
- 信封样式
- 磨砂玻璃效果
- 复古便签
- 糖果条纹
- 霓虹描边
- 日系手账风
- 水彩晕染
{% endfolding %}

这些样式的设计灵感来自原 Hexo 博客 Butterfly 标签体系——尤其 Akilar 的糖果屋风格。

### 其他功能

- **相册**：三个相册卡片，第一个收录动漫图片
- **侧边栏折叠**：右上角按钮一键收起，只留图标导航
- **移动端适配**：768px / 480px 两档响应式，抽屉式导航
- **中英切换**：I18nContext 组件级语言切换
- **天气特效**：RainCanvas 粒子雨系统，渐变色切换
- **B 站追番**：JSONP 拉取追番数据
- **留言墙**：信封投递交互，localStorage 本地存储
- **旧链接兼容**：旧 `.html` URL → 新路由 301 重定向

---

## 技术栈

| 环节 | 工具 |
|------|------|
| 框架 | Vite + React 19 + TypeScript |
| 路由 | react-router-dom v7 |
| 内容管线 | Markdown → Python build_content.py → JSON → React |
| 设计 | OpenAI Codex（视觉框架）+ Cursor（碎片页面） |
| 体系开发 | Claude Code + DeepSeek V4 Pro |
| 识图辅助 | 千问 VL 模型 |
| 手机操控 | cc-connect + 微信 |
| 配置管理 | CC Switch |
| 部署平台 | Vercel |
| 图片存储 | 腾讯云 COS |
| MCP 组件 | shadcn MCP + React Bits 520+ 组件 |

---

## 辅助工具链

| 工具 | 用途 |
|------|------|
| cc-connect + 微信 | 手机远程操控电脑写代码、发文件 |
| CC Switch | 多 AI 供应商一键切换、Skills 统一管理 |
| 千问 VL 模型 | 识图→文字描述，补上 DeepSeek 视觉盲区 |
| docx / pptx / pdf Skills | 自动生成实验报告和幻灯片 |

---

## 后续计划

{% checkbox checked green, 静态 HTML 搭建 — 第一阶段完成 %}
{% checkbox checked green, React SPA 迁移 — 第二阶段完成 %}
{% checkbox checked green, 文章迁移 — 22 篇文章已从 Hexo 博客完整迁移 %}
{% checkbox checked green, 说说迁移 — 5 条说说已迁移，带随机艺术效果 %}
{% checkbox checked green, MCP 接入 — React Bits 520+ 组件可用 %}
{% checkbox , 更多相册 — 生活碎片、旅行街景待填充 %}
{% checkbox , React Bits 动效 — 实际接入动画组件 %}

---

## 总结

{% note success %}
两天时间，两次重大迭代——第一天从零搭建了静态 HTML 版的完整个人创作屋，第二天把它升级为 React SPA 架构。

最让我惊喜的是：**Vibe Coding 让架构演进变得和搭积木一样简单**。不需要啃文档、不需要手写重复样板代码——只需要描述我想要什么，AI 助手们各司其职把它实现出来。

从纯 HTML 到 React SPA，从手动部署到 Vercel 自动构建，从零组件到 MCP 520+ 动画组件——每一步都是「描述想法 → AI 实现 → 审查迭代」的循环。

新站地址：{% span blue, https://tzyy11.vercel.app %}

欢迎来逛。
{% endnote %}
