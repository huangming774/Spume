---
title: OpenAI Codex 实战 —— 让 AI 帮我做 PPT 和写实验报告
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI自动化博客图片/Snipaste_2026-05-12_18-54-19.png
date: 2026-05-12 00:00:00
categories: AI工具
tags: [OpenAI Codex, VibeCoding, PPT, Word, 数据结构, AI编程]
description: 使用 OpenAI Codex 一次性生成技术演讲 PPT 和数据结构实验报告 Word 文档的全过程记录与填坑总结。
---

{% note info %}
**OpenAI Codex** 是 OpenAI 推出的 CLI + 云端编程智能体，支持 `/goal` 自主循环执行、并行处理和开源定制。这篇文章记录了我在同一天里用它完成了两项任务：一份 18 页的技术演讲 PPT 和一份包含 3 个完整实验的数据结构实验报告 Word 文档。

整个过程不是"一键生成"，而是 **人机协作、反复调试** 的真实写照——这也是 2026 年 VibeCoding 的典型工作方式。
{% endnote %}

---

## {% span blue, 一、实战一：制作 2026 VibeCoding 工具全景 PPT %}

### 任务描述

给 Codex 下达了一个需求：制作一份面向课堂分享的 PPT，介绍 2026 年主流的 10 款 VibeCoding 工具。

Codex 生成了一个完整的 Node.js 脚本 `build-deck.mjs`，使用 `@oai/artifact-tool` 的 presentation-jsx 引擎，以 JSX 声明式语法构建 18 页幻灯片。

### PPT 内容结构

{% timeline PPT 十八页内容结构, blue %}
<!-- timeline 封面 -->
标题"2026年主流 VibeCoding 工具"，列出 10 款年度焦点工具，标注适用场景为课堂分享与技术科普。
<!-- timeline 目录 -->
五大章节导航：概念定义 → 生态全景 → 10款工具 → 横向对比 → 选型与未来。
<!-- timeline 什么是 VibeCoding -->
Karpathy 2024 年提出，2026 年已演进为多智能体协同的软件工程范式。四大特征：全栈智能体化、云端异步、MCP 生态、跨会话记忆。
<!-- timeline 生态全景 -->
四大阵营：国际巨头（Claude Code / Antigravity / Codex）、中国本土（Trae / 通义灵码）、独立 IDE（Cursor / Windsurf）、生态平台（Copilot / Devin / Replit AI）。
<!-- timeline 10 款工具逐页介绍 -->
Claude Code、Trae、Antigravity、OpenAI Codex、Cursor、GitHub Copilot、Windsurf、Devin、Replit AI、通义灵码 —— 每页包含定位、亮点、适用人群、优缺点和典型场景。
<!-- timeline 横向对比总结 -->
一表看清 10 款工具的类型、自主能力、中文体验、适合人群和费用模式。
<!-- timeline 选型决策指南 -->
按角色推荐工具组合策略：IDE + CLI + 后台智能体"三件套"。
<!-- timeline 未来展望 + 致谢 -->
四个关键词（自主、协作、记忆、私有知识），以及感谢聆听。
{% endtimeline %}

### 生成效果

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI自动化博客图片/Snipaste_2026-05-12_18-56-56.png, alt=PPT生成效果截图 %}

### {% span red, 踩坑与修复 %}

Codex 生成的代码虽然结构完整，但在实际运行中遇到了 **4 个 bug**，需要逐一排查修复：

{% folding red, 点击展开Bug详情和修复方案 %}

#### Bug 1：Grid 布局 `rowSpan` 写错

{% label 报错 red %} `Grid could not place child within the available tracks`

**原因：** `definitionSlide` 和 `toolSlide` 的 info 面板列使用了 `rowSpan: 2`，但外层 Grid 只定义了 2 行，row 0 已被标题和标签占满，子元素尝试跨到 row 2 时越界。

**修复：** `definitionSlide` 的 info 面板应跨列而非跨行 → `columnSpan: 2`；`toolSlide` 的 info 面板不需要 span → 直接移除。

#### Bug 2：Windows 路径双盘符

{% label 报错 red %} `ENOENT: mkdir 'D:\D:\openai_codex\...'`

**原因：** 通过 `new URL(import.meta.url).pathname` 获取的路径在 Windows 上带前导 `/`（如 `/D:/openai_codex/...`），`path.resolve` 处理后产生 `D:\D:\...`。

**修复：** 改用 Node.js 标准的 `fileURLToPath(import.meta.url)`：
```javascript
import { fileURLToPath } from "node:url";
const workspaceDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
```

#### Bug 3：内容溢出

{% label 报错 orange %} `column overflowed slide 3 height by 86.8px` → 逐次缩小至 1.2px

**原因：** 4 个 infoPanel 的总高度超出了 definitionSlide 第二行的可用空间。

**修复：** 减小 `infoPanel` 的 padding（20→16）、内部 gap（10→8）、body 字号（22→20），并缩小 definitionSlide 列的 gap（14→6）。

#### Bug 4：Blob API 不兼容

{% label 报错 red %} `blob.arrayBuffer is not a function` → `Buffer.from received undefined`

**原因：** `@oai/artifact-tool` 的 `PresentationFile.exportPptx()` 返回的是自定义 Blob 对象（`{ data: Uint8Array, mime: string }`），而 `slide.export({ format: "png" })` 返回的是标准 Web Blob（有 `.arrayBuffer()` 方法）。两个 API 不一致。

**修复：** 兼容两种类型：
```javascript
const data = blob.data ?? new Uint8Array(await blob.arrayBuffer());
```

{% endfolding %}

### 最终成果

```
✅ 18 页幻灯片全部生成
📄 output.pptx
🖼️ 18 张 PNG 预览图 + Layout JSON
```

---

## {% span blue, 二、实战二：撰写数据结构实验报告 Word 文档 %}

### 任务描述

同一次对话中，Codex 还帮我完成了另一项任务：从数据结构课程的实验要求中**任选三个实验**，写出包含 **实验应用、算法设计、完整 C 语言代码、实验总结** 的正式报告，输出为 Word 文档。

Codex 使用 Python 的 `python-docx` 库编写了 `build_codex_test1.py` 脚本，生成格式规范的 `.docx` 文件。

### 三个实验概览

{% folding blue, 实验一：单链表学生信息管理系统 %}

**{% span blue, 实验应用 %}**

学生成绩管理系统是高校教务中最常见的数据管理场景。本实验使用单链表实现学生信息的增删改查，适用于小规模数据的链式存储管理。

**{% span blue, 算法设计 %}**
- 数据结构：带头结点的单链表，每个结点包含学号、姓名、成绩、指针域
- 核心操作：头插法/尾插法建表、按学号查找、按位置插入、按学号删除、遍历输出
- 时间复杂度：查找/插入/删除均为 O(n)，头插为 O(1)

**{% span blue, C 语言实现要点 %}**
```c
typedef struct Student {
    char id[12];
    char name[20];
    float score;
    struct Student *next;
} Student;
// 尾插法、查找、插入、删除、排序、输出共 6 个函数
```

**{% span blue, 实验总结 %}**
单链表的动态内存特性使其在频繁增删的场景中优于顺序表。本实验加深了对指针操作和链式存储的理解。

{% endfolding %}

{% folding green, 实验二：哈夫曼编码与解码系统 %}

**{% span green, 实验应用 %}**

哈夫曼编码是数据压缩领域的经典算法，广泛应用于 ZIP 压缩、JPEG 图像编码、MP3 音频编码等场景。本实验实现完整的编码-解码流程。

**{% span green, 算法设计 %}**
- 数据结构：哈夫曼树（二叉树）、最小堆（优先队列）、编码表（哈希映射）
- 构建流程：统计字符频率 → 建最小堆 → 反复合并最小两结点 → 递归生成编码
- 核心操作：`buildHuffmanTree()`、`generateCodes()`、`encode()`、`decode()`
- 时间复杂度：建堆 O(nlogn)，编码生成 O(n)

**{% span green, C 语言实现要点 %}**
```c
typedef struct HuffmanNode {
    char ch;
    int freq;
    struct HuffmanNode *left, *right;
} HuffmanNode;
// 含最小堆、建树、生成编码、编码文件、解码文件
```

**{% span green, 实验总结 %}**
哈夫曼编码展示了贪心算法在最优前缀码构造中的精妙应用，是理解无损压缩原理的绝佳入门案例。

{% endfolding %}

{% folding orange, 实验三：校园最短路径导航系统 %}

**{% span orange, 实验应用 %}**

校园导航是图论最短路径算法的典型应用场景。本实验以校园建筑为结点、道路为边，实现两地之间的最短路径查询与可视化路径输出。

**{% span orange, 算法设计 %}**
- 数据结构：邻接矩阵存储带权无向图
- 核心算法：Dijkstra 算法（单源最短路径）+ Floyd 算法（全源最短路径）
- 辅助功能：路径回溯（`prev[]` 数组）、路径打印（递归输出经过的建筑名称）
- Dijkstra 时间复杂度：O(n²)，适合结点数 ≤ 100 的场景

**{% span orange, C 语言实现要点 %}**
```c
#define MAXV 20
typedef struct {
    char name[30];
    int edges[MAXV][MAXV];
    int n; // 实际结点数
} CampusGraph;
// Dijkstra 最短路径 + 路径回溯 + 路径打印
```

**{% span orange, 实验总结 %}**
Dijkstra 算法是图论中最优雅的经典算法之一。通过实际地图数据的导入，代码直接从"纸上算法"变成了可用的导航工具。

{% endfolding %}

### 文档格式

{% checkbox checked blue, 封面页（标题 + 元信息表） %}
{% checkbox checked blue, 目录页（三个实验的导航） %}
{% checkbox checked blue, 每实验含：实验应用 / 算法设计 / C 代码 / 结果说明 / 实验总结 %}
{% checkbox checked green, C 代码通过 gcc -fsyntax-only -std=c99 语法检查 %}
{% checkbox checked green, 微软雅黑标题 + 宋体正文 + Consolas 代码，1.5 倍行距 %}

---

## {% span blue, 三、经验与反思 %}

### {% span coral, 1. VibeCoding 不是"一键生成" %}

无论是 PPT 的 4 个 bug 修复，还是 Word 文档的结构设计，整个过程都是 **人机协作**：

- AI 负责：生成初版代码、搭建框架、填充内容
- 人负责：定义目标、审查结果、修复边界情况、验证输出

这正是 Karpathy 所说的"从逐行写代码，转向描述意图、审查生成结果"。

### {% span coral, 2. 技术栈选择很关键 %}

{% note success %}
PPT 使用 `@oai/artifact-tool` 的 JSX 声明式语法，代码量约 960 行但结构清晰、可维护性强。Word 使用 `python-docx`，API 直观，适合程序化生成格式化文档。两者都是各自领域的优秀选择。
{% endnote %}

### {% span coral, 3. 环境差异不容忽视 %}

Windows 路径处理、不同版本库的 API 差异、CSS Grid 布局的隐式行限制——这些问题 AI 生成的代码不一定能预判到，需要开发者有调试能力和底层理解。

---

## {% span blue, 总结 %}

{% note primary %}
这次实战让我深刻体会到：**2026 年的 VibeCoding，本质上是"AI 负责广度（快速生成框架和基础代码），人负责深度（审查、调试、优化、验证）"**。

Codex 的 `/goal` 自主循环能力让它在处理这类"生成型"任务时表现出色——定义好输入输出的规格，它就能持续迭代直到完成。而人的价值在于：判断方向是否正确、修复 AI 的盲区、把关最终质量。

两个项目都在半小时内完成了从需求到交付的全过程，这在传统开发模式下是不可想象的。
{% endnote %}
