---
title: 71 个品牌设计系统一键注入——awesome-design-md 做成 Claude Code Skill
date: 2026-05-18 23:20:00
tags: [Claude Code, Skill, DESIGN.md, 设计系统, 前端, 开源]
categories: [开发工具]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/awesome-design-md-skill-cover.png
---

{% note info %}
项目地址：[github.com/VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md) — 72,000+ Stars，MIT 许可证
{% endnote %}

---

## 这是什么？

**awesome-design-md** 是 VoltAgent 团队维护的开源仓库，把 **71 个知名产品** 的设计系统（配色、字体、间距、组件样式、阴影、响应式规则）提炼成 AI 能直接阅读的 **DESIGN.md** 文件。

你选一个品牌，把它的 `DESIGN.md` 放进项目根目录，然后对 Claude Code 说"请根据 DESIGN.md 的设计规范生成界面"——AI 就会按那个品牌的风格来写代码。

---

## 覆盖的品牌（71 个，9 大类）

{% folding green, 点击展开全部品牌列表 %}

### AI & LLM
Claude、Cohere、Cursor、ElevenLabs、Minimax、Mistral AI、Ollama、OpenCode AI、Replicate、RunwayML、Together AI、VoltAgent、xAI

### 开发者工具 & IDE
Expo、Lovable、Raycast、Superhuman、Vercel、Warp

### 设计 & 创意工具
Airtable、Clay、Figma、Framer、Miro、Webflow

### 后端、数据库 & DevOps
ClickHouse、Composio、HashiCorp、MongoDB、PostHog、Sanity、Sentry、Supabase

### 生产力 & SaaS
Cal.com、Intercom、Linear、Mintlify、Notion、Resend、Zapier

### 金融科技 & 加密货币
Binance、Coinbase、Kraken、Mastercard、Revolut、Stripe、Wise

### 电商 & 零售
Airbnb、Meta、Nike、Shopify、Starbucks

### 消费科技 & 媒体
Apple、HP、IBM、NVIDIA、Pinterest、PlayStation、SpaceX、Spotify、The Verge、Uber、Vodafone、WIRED

### 汽车
BMW、BMW M、Bugatti、Ferrari、Lamborghini、Renault、Tesla

{% endfolding %}

---

## DESIGN.md 的结构——9 大模块

每个品牌的 DESIGN.md 都遵循 Google Stitch 引入的标准格式，包含 9 个章节：

{% folding blue, 9 大模块详解 %}

| # | 模块 | 内容示例 |
|---|------|------|
| 1 | Visual Theme & Atmosphere | "极简高对比度，冷色调技术感" |
| 2 | Color Palette & Roles | `--accent: #5E6AD2` `--bg-primary: #000000` 语义色板 |
| 3 | Typography Rules | 字体家族 + Display/H1/H2/Body/Caption 字号层级表 |
| 4 | Component Stylings | 按钮/卡片/输入框 的 default/hover/active/disabled/focus 五态样式 |
| 5 | Layout Principles | 间距比例尺 4/8/12/16/24/32/48/64px，12 列网格 |
| 6 | Depth & Elevation | 0 级（平面）→ 3 级（模态弹窗）四层阴影系统 |
| 7 | Do's and Don'ts | "✅ 用语义色名，不要直接写 hex" 之类设计护栏 |
| 8 | Responsive Behavior | 断点 <640 / 640-1024 / >1024，触控目标 ≥44px |
| 9 | Agent Prompt Guide | 快速颜色参考 + 现成 prompt 模板 |

{% endfolding %}

---

## 把它做成了 Claude Code Skill

为了不用每次都去 GitHub 翻仓库，我把它封装成了一个 **Claude Code Skill**，存储在本地技能库中。

### Skill 做了什么

{% tip info %}
Skill 文件：`C:\Users\lenovo\.claude\skills\design-md\SKILL.md`（183 行），已自动加载到 Claude Code 的技能列表。
{% endtip %}

核心能力：

1. {% span blue, 品牌检索 %} — 71 个品牌按 9 大类组织，可以说"给我 Tesla 风格"直接定位
2. {% span green, 一键应用 %} — 从 GitHub clone 仓库 → 复制 DESIGN.md 到项目根目录
3. {% span red, 设计护栏 %} — 自动遵循 DESIGN.md 中的 9 大模块规则生成 UI，不跑偏
4. {% span purple, 视觉预览 %} — 每个品牌附带 `preview.html` + `preview-dark.html`，可在浏览器中直接看色板、字号比例、按钮样式

### 触发方式

当在对话中说类似以下内容时，Skill 自动激活：

```
"做成 Linear 风格"
"我想要 Apple 那种设计"
"帮我参考 Stripe 的配色"
"有没有适合后台管理系统的设计规范"
```

---

## 使用流程

### 第一步：选品牌

告诉 Claude 你想要哪个品牌的风格，Skill 会从 71 个品牌中匹配。

### 第二步：部署 DESIGN.md

```bash
git clone --depth 1 https://github.com/VoltAgent/awesome-design-md.git /tmp/awesome-design-md
cp /tmp/awesome-design-md/design-md/<brand>/DESIGN.md ./DESIGN.md
```

### 第三步：告诉 AI 用它

之后任何涉及 UI 的对话，Claude 都会自动参考项目根目录的 `DESIGN.md`：

```
"基于 DESIGN.md 的设计规范，帮我重构首页的 Hero 区域"
"把导航栏改成 DESIGN.md 里定义的组件样式"
"给这个页面应用 DESIGN.md 的配色方案"
```

---

## 实战演示

{% note success %}
**场景：** 一个个人主页项目，想要 Linear 那种极简精准的审美风格。
{% endnote %}

```bash
# 1. 复制 Linear 的 DESIGN.md
cp /tmp/awesome-design-md/design-md/linear.app/DESIGN.md ./DESIGN.md

# 2. 告诉 Claude
"请根据项目根目录的 DESIGN.md，重构整个页面的配色和组件样式"
```

Claude 会读取 Linear 的设计规范：
- {% span blue, 配色 %} — 极简黑白，紫色点缀 `#5E6AD2`
- {% span blue, 字体 %} — Inter Weight 510，紧凑字间距
- {% span blue, 组件 %} — 微妙的边框、精确的圆角、克制的阴影层级
- {% span blue, 布局 %} — 精准对齐、清晰信息层级

然后输出符合 Linear 风格的代码。

---

## 为什么值得用

{% note primary %}
没有 DESIGN.md 时，AI 对"好看"的理解是模糊的——你只能说"做大一点""颜色亮一点"这种抽象的反馈。有了 DESIGN.md，AI 有了具体的数值参考：字号多少 px、色值是什么 hex、间距用哪个 scale 格、阴影用哪个层级。
{% endnote %}

核心收益：

| 之前 | 之后 |
|------|------|
| "这个卡片好看点" | "按 DESIGN.md §4 Component Stylings 中的 Card 规范" |
| "字体大一点" | "按 DESIGN.md §3 Typography 中的 H2 级别" |
| "颜色不对" | "按 DESIGN.md §2 Color Palette 中的 accent 色值" |
| 每次重新描述设计 | DESIGN.md 一次写入，全局生效 |

---

## 参考链接

- [awesome-design-md GitHub 仓库](https://github.com/VoltAgent/awesome-design-md) — 71 个品牌的 DESIGN.md 文件
- [DESIGN.md 协议介绍](https://ossinsight.io/blog/design-md-protocol-2026) — DESIGN.md 成为 GitHub 增长最快的设计标准
- [VoltAgent 官方文档](https://voltagent.dev/docs/ai-assistants/) — AI 助手集成文档
