---
title: 软件工程概论实验——DeepL语言翻译平台系统分析与设计
date: 2026-05-15 15:00:00
tags: [软件工程, 系统分析与设计, UML建模, DeepL, 实验报告]
categories: [学习笔记]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/Snipaste_2026-05-15_14-44-14.png
description: 基于面向对象方法对DeepL翻译平台进行完整的系统分析与设计，涵盖用例建模、功能结构设计、数据库设计、规模估算等核心环节。
---

## 实验概述

{% note info no-icon %}
本实验是《软件工程概论》课程的综合性实验——**系统分析与设计**，总计 8 学时。通过选定 DeepL 语言翻译平台作为分析对象，运用面向对象的软件工程思想，完成从需求分析到详细设计的完整过程。
{% endnote %}

### 实验目的

1. **理解并掌握**面向对象的系统分析方法，包括用例建模、领域建模等技术
2. **理解并掌握**面向对象的系统设计方法，包括架构设计、模块设计、数据库设计等
3. **掌握** ProcessOn、EA、Visio 等 CASE 工具建立系统分析与设计的软件模型
4. **理解**需求分析和设计在软件开发过程中的核心地位和作用
5. **了解**书写软件需求规格说明书和软件设计说明书的相关规范

---

## 实验对象：DeepL 翻译平台

{% note primary no-icon %}
DeepL 是一款基于深度学习神经网络技术（Transformer 架构）的高质量在线翻译服务，支持数十种语言互译，并提供文档翻译、网页翻译、API 服务、写作辅助（DeepL Write）等多种功能。
{% endnote %}

核心功能介绍：

{% folding blue, 文本翻译 %}
支持用户输入文本，选择源语言和目标语言（支持自动语言检测），实现在线实时翻译，并提供翻译结果的复制、朗读、词典查询等交互操作。
{% endfolding %}

{% folding green, 文档翻译 %}
支持上传 PDF、DOCX、PPTX 等格式文档，保持原始格式进行全文翻译，并提供下载功能。
{% endfolding %}

{% folding cyan, API 服务 %}
为开发者提供 RESTful API 接口，支持将 DeepL 翻译能力集成到第三方应用程序中。
{% endfolding %}

{% folding orange, 词汇表管理 %}
允许用户创建和管理自定义术语词汇表，确保特定术语的翻译一致性。
{% endfolding %}

{% folding purple, DeepL Write %}
提供 AI 写作辅助功能，帮助用户改进语法、措辞和风格（目前支持英语和德语等主要语言）。
{% endfolding %}

---

## 系统分析——用例建模

### 参与者识别

{% folding blue, 系统参与者列表 %}

| 参与者 | 类型 | 描述 |
|--------|------|------|
| {% span red, 普通用户 (Visitor) %} | 主要参与者 | 未登录访客，可使用基本翻译功能，有字符限制 |
| {% span blue, 注册用户 (Registered User) %} | 主要参与者 | 已登录用户，可使用翻译历史、词汇表等功能 |
| {% span purple, 付费订阅用户 (Subscriber) %} | 主要参与者 | DeepL Pro 用户，享有无限翻译、API 访问等高级功能 |
| {% span orange, 团队管理员 (Team Admin) %} | 主要参与者 | 管理团队订阅、成员权限和翻译记忆库 |
| {% span green, 第三方开发者 (Developer) %} | 主要参与者 | 通过 API 集成 DeepL 翻译服务的开发人员 |
| 系统管理员 (Admin) | 次要参与者 | 负责系统运维、监控、用户管理 |
| 支付系统 (Payment System) | 外部系统 | 处理订阅支付的外部支付网关 |

{% endfolding %}

### 顶层用例图说明

顶层用例包含 **11 个核心用例**：

{% checkbox checked blue, UC01 - 文本翻译 %}
{% checkbox checked blue, UC02 - 文档翻译 %}
{% checkbox checked blue, UC03 - 网页翻译 %}
{% checkbox checked blue, UC04 - API调用 %}
{% checkbox checked blue, UC05 - 词汇表管理 %}
{% checkbox checked blue, UC06 - 写作辅助 %}
{% checkbox , UC07 - 用户注册与登录 %}
{% checkbox , UC08 - 订阅管理 %}
{% checkbox , UC09 - 翻译历史管理 %}
{% checkbox , UC10 - 团队管理 %}
{% checkbox , UC11 - 系统管理 %}

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-use-case-diagram.png" alt="图4-1 DeepL翻译平台顶层用例图" style="width:92%">

### 核心用例描述

{% folding cyan, UC01 文本翻译用例 %}
| 项目 | 内容 |
|------|------|
| 用例编号 | UC01 |
| 参与者 | 普通用户、注册用户、订阅用户 |
| 前置条件 | 用户已访问 DeepL 翻译页面 |
| 后置条件 | 系统返回翻译结果，记录翻译历史 |
| **基本事件流** | |
| 1 | 用户输入待翻译文本 |
| 2 | 系统自动检测源语言 |
| 3 | 用户选择目标语言 |
| 4 | 系统实时调用翻译引擎 |
| 5 | 系统显示翻译结果 |
| 6 | 用户可复制、朗读、替换同义词 |
| **异常事件流** | |
| E1 | 文本超过字符限制 → 提示注册或升级 |
| E2 | 翻译引擎服务异常 → 提示稍后重试 |
| 优先级 | {% label 高 red %}（核心功能） |
{% endfolding %}

{% folding cyan, UC02 文档翻译用例 %}
| 项目 | 内容 |
|------|------|
| 用例编号 | UC02 |
| 参与者 | 注册用户、订阅用户 |
| 前置条件 | 用户已登录系统 |
| 后置条件 | 系统返回翻译后的文档下载链接 |
| 基本事件流 | 上传文档 → 验证格式 → 选择语言 → 翻译引擎处理 → 保留原始格式 → 下载 |
| 异常 | 格式不支持、文件过大、翻译超时 |
| 优先级 | {% label 高 red %} |
{% endfolding %}

{% folding cyan, UC04 API调用用例 %}
| 项目 | 内容 |
|------|------|
| 用例编号 | UC04 |
| 参与者 | 第三方开发者（需 API 密钥） |
| 前置条件 | 开发者已注册并获取有效的 API 密钥 |
| 后置条件 | 系统返回 JSON 格式翻译结果，更新 API 使用量 |
| 异常 | 401 Unauthorized / 429 Too Many Requests / 400 Bad Request / 500 Internal Server Error |
| 优先级 | {% label 高 red %} |
{% endfolding %}

---

## 系统功能结构设计

系统分为 **6 个一级功能模块**：

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-function-structure.jpg" alt="图4-2 系统功能层次结构图" style="width:92%">

{% folding blue, 文本翻译模块（核心） %}
系统的**核心功能模块**。负责接收用户输入的文本，通过神经网络翻译引擎进行实时翻译。支持多种语言互译，具备源语言自动检测、翻译结果词典查询、同义词替换建议等增强功能。

{% progress 100%, blue, 核心模块 %}
{% endfolding %}

{% folding green, 文档翻译模块 %}
处理用户上传的文档文件，解析文档结构后进行内容翻译，同时保持原文档的排版格式。支持 PDF、DOCX、PPTX 等常见办公文档格式。

{% progress 90%, green, 重要模块 %}
{% endfolding %}

{% folding purple, 写作辅助模块（DeepL Write） %}
基于 AI 语言模型，对用户输入的文本进行语法检查、措辞优化和风格调整。支持多种写作风格（正式、非正式、商务、学术等），并提供逐条修改建议。

{% progress 75%, purple, DeepL Write %}
{% endfolding %}

{% folding orange, 用户服务模块 %}
管理用户账户全生命周期，包括注册、登录、个人信息管理、订阅计划管理、翻译历史记录查询、自定义词汇表维护等功能。

{% progress 85%, orange, 基础服务 %}
{% endfolding %}

{% folding cyan, API 服务模块 %}
为第三方开发者提供标准化的 RESTful API 接口，包括 API 密钥管理、请求认证、频率限制、用量计费、开发者文档等功能。

{% progress 80%, cyan, 开放能力 %}
{% endfolding %}

{% folding default, 系统管理模块 %}
提供后台管理功能，包括用户审核、翻译引擎运行状态监控、系统日志收集与告警、业务数据统计与分析等运维管理功能。

{% progress 70%, green, 运维支撑 %}
{% endfolding %}

---

## 系统模块结构设计

### 分层架构设计

{% note success no-icon %}
DeepL 翻译平台采用**分层架构（Layered Architecture）**设计模式，系统自上而下分为四层，各层之间通过接口进行通信，遵循**依赖倒置原则（DIP）**。
{% endnote %}

```
┌─────────────────────────────────┐
│     表现层 (Presentation)        │  ← Web前端、移动端界面
├─────────────────────────────────┤
│    业务逻辑层 (Business Logic)    │  ← 翻译业务处理、用户服务
├─────────────────────────────────┤
│       服务层 (Service)           │  ← 翻译引擎、AI写作引擎
├─────────────────────────────────┤
│     数据访问层 (Data Access)      │  ← MySQL、Redis、对象存储
└─────────────────────────────────┘
```

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-layered-architecture.png" alt="图4-3 系统分层架构图" style="width:92%">

### 核心实体类设计

{% folding green, 六大核心实体类 %}

{% span blue, (1) User（用户类）%}
- 属性：userId, username, email, passwordHash, subscriptionType, createdAt
- 方法：register(), login(), updateProfile(), upgradeSubscription()

{% span blue, (2) Translation（翻译记录类）%}
- 属性：translationId, userId, sourceText, translatedText, sourceLang, targetLang
- 方法：create(), getHistory(), delete()

{% span blue, (3) Glossary（词汇表类）%}
- 属性：glossaryId, userId, name, sourceLang, targetLang, entries[]
- 方法：create(), addEntry(), removeEntry(), applyToTranslation()

{% span blue, (4) Document（文档类）%}
- 属性：documentId, userId, originalFilename, fileSize, fileType, translatedFileUrl, status
- 方法：upload(), translate(), download()

{% span blue, (5) Subscription（订阅类）%}
- 属性：subscriptionId, userId, planType, startDate, endDate, status
- 方法：create(), renew(), cancel(), checkValidity()

{% span blue, (6) ApiKey（API密钥类）%}
- 属性：apiKeyId, userId, keyValue, quotaLimit, quotaUsed, isActive
- 方法：generate(), validate(), revoke(), updateQuota()

{% endfolding %}

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-class-diagram.png" alt="图4-4 系统核心设计类图" style="width:92%">

---

## 模块详细设计

### 文本翻译模块——活动图说明

1. 用户在输入框中输入待翻译文本
2. 系统自动检测源语言（或用户手动选择）
3. 用户选择目标语言
4. 系统调用翻译引擎（同时匹配词汇表）
5. 系统返回翻译结果，保存翻译历史

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-text-translate-activity.png" alt="图4-5 文本翻译活动图" style="width:92%">

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-synonym-flow.png" alt="图4-5-2 同义词替换流程图" style="width:92%">

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-text-translate-sequence.png" alt="图4-6 文本翻译顺序图" style="width:92%">

### 文档翻译模块——程序流程

```
开始 → 用户上传文档 → 格式验证 → 文档解析
  → 调用文档翻译引擎 → 生成翻译文档 → 保留原始格式
  → 提供下载链接 → 结束
```

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-document-translate-flow.png" alt="图4-7 文档翻译程序流程图" style="width:92%">

### API调用模块——程序流程

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-api-activity.png, alt=图4-8 API调用活动图, width=100%}

---

## 数据库结构设计

系统采用 **MySQL 8.0** 作为关系型数据库，使用 **Redis** 进行会话和缓存管理，使用**对象存储（如 AWS S3）**存储文档文件。

### 数据库 E-R 图核心实体

{% note info no-icon %}
核心实体：User（用户）、Translation（翻译记录）、Glossary（词汇表）、GlossaryEntry（词汇表条目）、Document（文档）、Subscription（订阅）、ApiKey（API密钥）、Payment（支付记录）
{% endnote %}

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-er-diagram.jpg" alt="图4-9 数据库E-R图" style="width:92%">

### 核心数据库表

{% folding blue, t_user（用户表） %}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| user_id | BIGINT UNSIGNED | PK, AUTO_INCREMENT | 用户唯一标识 |
| username | VARCHAR(50) | NOT NULL, UNIQUE | 用户名 |
| email | VARCHAR(255) | NOT NULL, UNIQUE | 邮箱地址 |
| password_hash | VARCHAR(255) | NOT NULL | bcrypt 密码哈希 |
| subscription_type | ENUM('free','starter','advanced','ultimate') | NOT NULL | 订阅类型 |
| created_at | DATETIME | NOT NULL | 创建时间 |
| updated_at | DATETIME | NOT NULL | 更新时间 |
{% endfolding %}

{% folding green, t_translation（翻译记录表） %}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| translation_id | BIGINT UNSIGNED | PK | 翻译记录ID |
| user_id | BIGINT UNSIGNED | FK→t_user, NULL | 用户ID |
| source_text | TEXT | NOT NULL | 源语言文本 |
| translated_text | TEXT | NOT NULL | 翻译后文本 |
| source_lang | VARCHAR(10) | NOT NULL | 源语言代码 |
| target_lang | VARCHAR(10) | NOT NULL | 目标语言代码 |
| char_count | INT UNSIGNED | NOT NULL | 源文本字符数 |
| created_at | DATETIME | NOT NULL | 翻译时间 |

{% span green, 索引：%} idx_user_id, idx_created_at, idx_user_lang
{% endfolding %}

{% folding cyan, t_glossary（词汇表） %}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| glossary_id | BIGINT UNSIGNED | PK | 词汇表ID |
| user_id | BIGINT UNSIGNED | FK→t_user | 所属用户 |
| name | VARCHAR(200) | NOT NULL | 词汇表名称 |
| source_lang | VARCHAR(10) | NOT NULL | 源语言 |
| target_lang | VARCHAR(10) | NOT NULL | 目标语言 |
{% endfolding %}

{% folding orange, t_subscription（订阅表） %}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| subscription_id | BIGINT UNSIGNED | PK | 订阅ID |
| user_id | BIGINT UNSIGNED | FK→t_user, UNIQUE | 用户ID |
| plan_type | ENUM('starter','advanced','ultimate','team') | NOT NULL | 订阅计划 |
| billing_cycle | ENUM('monthly','annual') | NOT NULL | 计费周期 |
| start_date | DATE | NOT NULL | 开始日期 |
| end_date | DATE | NOT NULL | 结束日期 |
| auto_renew | BOOLEAN | DEFAULT TRUE | 自动续费 |
{% endfolding %}

{% folding purple, t_api_key（API密钥表） %}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| api_key_id | BIGINT UNSIGNED | PK | 密钥ID |
| user_id | BIGINT UNSIGNED | FK→t_user | 所属用户 |
| key_value | VARCHAR(64) | NOT NULL, UNIQUE | 加密存储 |
| monthly_quota | BIGINT UNSIGNED | NOT NULL | 月配额 |
| quota_used | BIGINT UNSIGNED | DEFAULT 0 | 已使用配额 |
{% endfolding %}

---

## 软件过程模型选择

{% note success no-icon %}
本系统开发拟采用**增量模型（Incremental Model）**结合**敏捷开发实践（Scrum 框架）**的混合过程模型。
{% endnote %}

### 选择理由

1. **核心功能清晰可分**：翻译功能可按优先级划分为多个增量，首批实现 MVP 快速上线
2. **市场需求反馈驱动**：翻译工具市场竞争激烈，需要快速推出可用产品获取用户反馈
3. **AI 模型的持续迭代特性**：神经网络模型需要持续训练和优化，增量模型利于集成改进版本
4. **风险分散**：早期增量可验证核心翻译质量和技术架构的可行性
5. **团队协作效率**：结合 Scrum 框架，按 2-4 周 Sprint 周期迭代开发

### 增量划分方案

| 增量 | 交付内容 | 周期 | 核心功能 |
|------|----------|------|----------|
| {% span blue, 增量1 %} | MVP 文本翻译核心 | 8 周 | 基础翻译、语言检测、Web 前端 |
| {% span green, 增量2 %} | 用户系统与文档翻译 | 6 周 | 注册/登录、文档翻译、翻译历史 |
| {% span purple, 增量3 %} | API 服务与词汇表 | 6 周 | RESTful API、API 密钥、词汇表 |
| {% span orange, 增量4 %} | 付费订阅与团队管理 | 6 周 | 订阅计划、支付集成、团队管理 |
| {% span cyan, 增量5 %} | DeepL Write | 6 周 | AI 写作辅助、语法检查、风格切换 |
| {% span red, 增量6 %} | 性能优化与扩展 | 4 周 | 性能优化、CDN、多区域支持 |

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/deepl-gantt-chart.png" alt="图4-10 增量开发计划甘特图" style="width:92%">

---

## 系统规模估算

{% note warning no-icon %}
软件规模估算是软件项目管理的关键环节。本实验采用**代码行技术（LOC）**和**功能点技术（Function Point）**两种方法对 DeepL 翻译平台进行规模估算。
{% endnote %}

### 代码行技术（LOC）估算

| 模块 | 预计 LOC | 开发语言 |
|------|----------|----------|
| 文本翻译核心引擎 | 25,000 - 35,000 | Python/C++ |
| Web 前端界面 (React) | 15,000 - 22,000 | TypeScript/JSX |
| 用户服务模块 | 8,000 - 12,000 | Java/Python |
| 文档翻译模块 | 10,000 - 15,000 | Python |
| API 网关服务 | 6,000 - 10,000 | Java/Go |
| 词汇表管理模块 | 5,000 - 8,000 | Python |
| 订阅与支付模块 | 5,000 - 8,000 | Java |
| 写作辅助 (DeepL Write) | 12,000 - 18,000 | Python |
| 后台管理系统 | 8,000 - 12,000 | TypeScript |
| 数据库与缓存层 | 3,000 - 5,000 | SQL/配置 |
| 测试代码 | 12,000 - 18,000 | 多种语言 |
| 基础设施建设 | 4,000 - 6,000 | YAML/Terraform |
| **合计** | **约 140,000 LOC** | — |

{% progress 100%, blue, 总代码行数：~140,000 LOC %}

根据 COCOMO II 模型，预计总人月约 **280 人月**。若团队规模为 15 人，开发周期约 **18-19 个月**。

### 功能点技术（FP）估算

#### （1）未调整功能点（UFP）计算

| 功能类型 | 复杂度 | 数量 | 权重 | 贡献值 |
|----------|--------|------|------|--------|
| 外部输入 (EI) | 中 | 10 | 4 | 40 |
| 外部输出 (EO) | 中 | 8 | 5 | 40 |
| 外部查询 (EQ) | 低 | 6 | 3 | 18 |
| 内部逻辑文件 (ILF) | 中 | 8 | 10 | 80 |
| 外部接口文件 (EIF) | 低 | 3 | 5 | 15 |

> **UFP = 193 FP**（未调整功能点 Unadjusted Function Point）

#### （2）技术复杂度因子（TCF）计算

{% folding cyan, 14项通用系统特性（GSC）评估 %}

| GSC | 技术因素 | 评分 | 说明 |
|-----|----------|------|------|
| F1 | 数据通信 | 5 | 高度依赖网络通信，支持实时翻译 |
| F2 | 分布式数据处理 | 4 | 翻译引擎可分布式部署 |
| F3 | 性能 | 5 | 翻译响应时间要求高 |
| F4 | 高负载配置 | 4 | 需支持高并发翻译请求 |
| F5 | 事务速率 | 4 | API 请求速率高 |
| F9 | 复杂处理 | 5 | NMT 翻译引擎是高度复杂的 AI 处理逻辑 |

TCF = 0.65 + 0.01 × DI ≈ **1.18**

{% endfolding %}

#### （3）最终功能点

{% note primary no-icon %}
**FP = UFP × TCF = 193 × 1.18 ≈ 228 FP**
{% endnote %}

### 估算对比分析

{% note warning no-icon %}
功能点技术估算偏低，主要原因是 DeepL 的核心 AI 翻译引擎属于高度复杂的算法密集型软件，功能点技术对算法复杂度的考量有限。在实际项目管理中，**建议以代码行估算（约 140,000 LOC）为主要参考**。
{% endnote %}

---

## 实验总结与体会

通过本次实验，我对软件工程中系统分析与设计的方法和技术有了更深入的理解：

**{% span red, 01 系统分析是软件开发的基石 %}**——只有充分理解用户需求、梳理业务流程、明确功能边界，才能为后续设计奠定坚实基础。用例建模技术是捕获和表达功能需求的有效工具。

**{% span blue, 02 面向对象方法的优势 %}**——通过用例图、类图、活动图、顺序图等 UML 模型，能够从不同视角全面描述系统。封装、继承和多态特性更符合现实世界的建模需求。

**{% span green, 03 分层架构的设计思想 %}**——将系统分为表现层、业务逻辑层、服务层和数据访问层，有效实现了关注点分离，降低了模块间的耦合度，提高了系统的可维护性和可扩展性。

**{% span purple, 04 数据库设计的关键性 %}**——数据库结构设计直接影响到系统的性能和可扩展性。需要考虑数据完整性约束、索引优化、范式设计等因素。

**{% span orange, 05 过程模型对项目的影响 %}**——DeepL 这类 AI 驱动产品适合增量模型，可以快速验证核心价值，逐步完善功能体系，降低技术风险和市场风险。

**{% span cyan, 06 规模估算的实践意义 %}**——软件规模估算既是科学也是艺术。不同估算方法有各自的适用场景和局限性，实际项目中应综合运用多种方法。

### 不足之处

{% folding red, 自我反思 %}

1. 对 DeepL 翻译平台的内部技术实现（特别是神经网络翻译模型的具体架构）的了解有限，分析深度可能不够充分
2. 由于缺乏真实用户调研数据，需求分析主要基于个人对系统的使用体验，可能与实际用户需求存在偏差

{% endfolding %}

---

## 参考文献

{% folding green, 参考文献与技术标准 %}

| 编号 | 文献 |
|------|------|
| [1] | 张海藩, 吕云翔. 软件工程（第5版）[M]. 北京: 人民邮电出版社, 2018. |
| [2] | Roger S. Pressman. Software Engineering: A Practitioner's Approach (8th Edition)[M]. McGraw-Hill, 2014. |
| [3] | IFPUG. Function Point Counting Practices Manual (Release 4.3.1)[S]. 2010. |
| [4] | ISO/IEC 25010:2011. Systems and software engineering — System and software quality models[S]. 2011. |
| [5] | DeepL API Documentation[EB/OL]. https://www.deepl.com/docs-api, 2026. |

{% endfolding %}

---

{% note info no-icon %}
本文为《软件工程概论》课程实验的实验报告，基于面向对象方法对 DeepL 语言翻译平台进行了完整的系统分析与设计。通过本次实验，将软件工程的理论知识与实际系统分析设计相结合，加深了对课程内容的理解，也提升了软件系统分析和设计的实践能力。
{% endnote %}
