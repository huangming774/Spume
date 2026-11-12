---
title: Claude Code Skills 配置指南 —— 打造你的 AI 超能力工具箱

cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/2026.5.9/Snipaste_2026-05-09_11-09-08.png
date: 2026-05-09 00:00:00
categories: 教程
tags: [Claude Code, Skills, AI工具, 效率]
description: 详细介绍如何在 Claude Code 中配置和使用 Skills，打造专属的 AI 工作流工具箱。
---

{% note info %}
Claude Code 是 Anthropic 推出的命令行 AI 编程助手，其 **Skills（技能）** 系统是它的核心亮点之一。Skills 相当于给 AI 装上"专业插件"，让它在特定任务上表现更出色——无论是生成 Word 文档、处理 PDF、设计前端界面，还是管理 Hexo 博客，都可以通过对应的 Skill 一键调用。

这篇文章记录了我配置的 **19 个 Skills** 的分类、功能和使用体验。
{% endnote %}

---

## {% span blue, 一、什么是 Skills？ %}

Skills 是 Claude Code 的可扩展能力模块。每个 Skill 本质上是一个 Markdown 文件（`SKILL.md`），包含：

- **触发条件** —— 用户说什么时自动激活
- **工作流程** —— 分步骤的操作指令
- **输出格式** —— 统一的结果呈现模板

当你输入 `/skill-name` 或提及相关关键词，Claude Code 会自动加载该 Skill 并按其流程执行。


---

## {% span purple, 二、我的 Skills 全家福 %}

### {% span blue, 🛠 文档办公类 %}

| Skill | 功能 | 一句话总结 |
|-------|------|-----------|
| {% label docx blue %} | Word 文档创建/编辑 | 自动生成专业排版的 .docx 文件 |
| {% label xlsx green %} | 表格数据处理 | 打开、创建、清洗、分析表格文件 |
| {% label pdf red %} | PDF 全能操作 | 合并、拆分、加密、OCR、提取文字 |
| {% label pptx purple %} | 幻灯片制作 | 从零生成演示文稿或编辑现有 PPT |

{% note default no-icon %}
日常办公场景的文档需求基本全覆盖，告别手动排版。
{% endnote %}

### {% span purple, 🎨 设计与前端类 %}

| Skill | 功能 | 一句话总结 |
|-------|------|-----------|
| {% label frontend-design blue %} | 前端界面设计 | 生成有设计感、不落俗套的网页/组件 |
| {% label ui-ux-pro-max purple %} | UI/UX 设计智能 | 67 种风格、96 套配色、57 种字体搭配可选 |

{% note primary no-icon %}
这两个技能配合使用，能让 AI 生成的前端界面拥有真正的"设计感"。
{% endnote %}

### {% span green, 🔧 开发辅助类 %}

| Skill | 功能 | 一句话总结 |
|-------|------|-----------|
| {% label skill-creator orange %} | 创建/优化 Skills | 用 AI 造新的 AI 技能 |
| {% label claude-api blue %} | Claude API 开发 | 构建和调试 Anthropic SDK 应用 |
| {% label simplify green %} | 代码审查优化 | 自动检查代码的复用性、质量和效率 |
| {% label review blue %} | PR 审查 | 对 Pull Request 进行自动化审查 |
| {% label security-review red %} | 安全审查 | 检查当前分支的代码安全问题 |
| {% label init green %} | 项目初始化 | 自动生成 CLAUDE.md 项目文档 |

{% note success no-icon %}
从写代码、审代码到部署前的安全检查，一条龙服务。
{% endnote %}

### {% span orange, ⚙ 效率与工具类 %}

| Skill | 功能 | 一句话总结 |
|-------|------|-----------|
| {% label loop blue %} | 定时循环任务 | 每 N 分钟自动执行指定命令 |
| {% label find-skills green %} | 技能发现 | 搜索和安装新的 Skills |
| {% label fewer-permission-prompts green %} | 权限优化 | 减少重复的权限确认弹窗 |
| {% label update-config orange %} | 配置管理 | 修改 Claude Code 的 settings.json |

{% note warning no-icon %}
让工具本身更顺手，减少操作摩擦。
{% endnote %}

### 📝 自建 Skills

除了官方 Skills，我还自己创建了三个专属 Skill：

| Skill | 功能 | 训练来源 |
|-------|------|----------|
| {% label blog-skill blue %} | Hexo 博客管理 | 自建 |
| {% label blog-tag-skill purple %} | 博客美化标签 | {% span green, Akilar 博客 %} |
| {% label git-first green %} | Git 版本控制 | 自建 |

{% folding blue, blog-skill —— Hexo 博客管理 %}
这是我自己创建的第一个 Skill，专门管理 Hexo 博客。触发词包括"博客""文章""hexo""发布"，工作目录 `C:/tzyy`，部署目标 `https://github.com/bistutzyy/bistutzyy.github.io`。

**工作流程：**

{% checkbox checked green, 检查 Hexo 版本和 Git 配置 %}
{% checkbox checked blue, 统计本地文章数量 %}
{% checkbox checked cyan, 验证 .deploy_git 部署目录状态 %}
{% checkbox checked purple, 输出状态汇总表格 %}
{% checkbox checked orange, 提示可用的操作（写文章/改配置/部署） %}

{% endfolding %}

{% folding purple, blog-tag-skill —— 博客美化标签 %}
这是从 {% span green, Akilar 的博客 %}（[Akilarの糖果屋](https://akilar.top/posts/615e2dec/)）学习训练的 Skill，收录了 **42 个** Hexo Butterfly 标签插件，涵盖行内文字、便签块、折叠面板、进度条、时间轴、图表等 18 大类。

包含的关键规则：
- 所有标签的正确语法（避免构建报错）
- 白底博客颜色选择规则（避开浅色/低对比度颜色）
- 42 个标签的逐一测试验证结果

写博客时自动加载，让文章排版更丰富。

{% endfolding %}

{% folding green, git-first —— Git 版本控制 %}
这是一个**执行纪律类** Skill，确保所有文件修改操作都通过 Git 进行版本控制。触发词包括"修改""编辑""创建文件""重构"等涉及文件变更的操作。

**核心规则：**

{% checkbox checked blue, 修改前先 commit —— 编辑任何文件前检查 git status，有未提交变更先提交 %}
{% checkbox checked green, 关键节点提交 —— 多步骤任务在每个逻辑节点拆分 commit %}
{% checkbox checked cyan, 大任务拆小 commit —— 一个功能一个 commit，不攒到一起 %}
{% checkbox checked orange, 完成后确认 —— 任务结束主动询问用户是否需要提交 %}
{% checkbox checked purple, 拒绝提交敏感文件 —— .env、credentials、node_modules 等自动排除 %}

**设计初衷：** 之前执行任务时偶有误删或错误修改无法恢复的情况。有了 git-first，每一步都有记录可追溯，出问题随时 `git revert`。

{% endfolding %}


---

## {% span green, 三、如何创建自己的 Skill？ %}

{% folding blue, Step 1：使用 skill-creator %}
输入 `/skill-creator`，告诉它你想创建的 Skill 的功能，它会引导你完成整个创建流程。
{% endfolding %}

{% folding green, Step 2：确定核心要素 %}

{% checkbox , 名称 — 简洁、易记（如 blog-skill） %}
{% checkbox , 触发条件 — 用户说什么会激活它 %}
{% checkbox , 工作步骤 — 每个检查或操作的具体命令 %}
{% checkbox , 输出格式 — 统一的展示模板 %}

{% endfolding %}

{% folding cyan, Step 3：测试迭代 %}
让 skill-creator 帮你跑测试用例，根据实际效果反复优化，直到满意为止。
{% endfolding %}

{% folding purple, Step 4：打包发布 %}
```bash
python -m scripts.package_skill <path/to/skill-folder>
```
生成 `.skill` 文件后可分享给其他人安装。
{% endfolding %}

---

## {% span cyan, 四、Skills 存放位置 %}

所有 Skills 安装后存放在：

```
C:\Users\lenovo\.claude\skills\
├── blog-skill/        # 自建 - 博客管理
├── blog-tag-skill/    # 自建 - 美化标签
├── git-first/         # 自建 - 版本控制
├── docx/              # 文档
├── xlsx/              # 表格
├── pdf/               # PDF
├── pptx/              # 幻灯片
├── frontend-design/   # 前端设计
├── ui-ux-pro-max/     # UI/UX
├── skill-creator/     # 技能创建器
├── claude-api/        # API 开发
├── simplify/          # 代码优化
├── review/            # PR 审查
├── security-review/   # 安全审查
├── init/              # 项目初始化
├── loop/              # 定时任务
├── find-skills/       # 技能发现
├── fewer-permission-prompts/  # 权限优化
└── update-config/     # 配置管理
```

---

## {% span orange, 五、使用感受 %}

Skills 系统让 Claude Code 从一个"通用 AI 助手"变成了一个"可定制的工作台"：

{% checkbox checked green, 专业性提升 — 处理特定文件格式（docx/pdf/xlsx）时不再需要临时摸索 %}
{% checkbox checked blue, 效率翻倍 — 博客更新从打开编辑器 → 写 Markdown → 手动 git push → 变成一句话的事 %}
{% checkbox checked cyan, 可扩展性 — 不满意内置 Skill？用 skill-creator 自己造一个 %}
{% checkbox checked purple, 减少重复劳动 — loop 可以让 AI 定时检查部署状态、PR 进度等 %}

{% tip success %}
如果你也在用 Claude Code，强烈建议花点时间配置适合自己工作流的 Skills。
{% endtip %}

---

## {% span red, 参考资料 %}

- [Claude Code 官方文档](https://docs.anthropic.com/en/docs/claude-code)
- [Skill Creator 使用指南](https://docs.anthropic.com/en/docs/claude-code/skills)
- [Akilarの糖果屋 — Butterfly 标签插件](https://akilar.top/posts/615e2dec/)（blog-tag-skill 训练来源）
- 本博客搭建教程：[github搭建博客笔记](/2025/01/03/github搭建博客笔记/)

---

*感谢 iflab 学长们的指导，感谢 Claude Code 让这一切变得简单。*
