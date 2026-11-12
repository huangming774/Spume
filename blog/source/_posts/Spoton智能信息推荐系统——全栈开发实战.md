---
title: Spoton智能信息推荐系统——全栈开发实战
date: 2026-05-28 23:30:00
tags: [Go, Vue3, Python, 全栈开发, 推荐系统]
categories: [项目实战]
---

最近在做大创项目，做了一个叫 **Spoton** 的智能信息推荐系统，简单来说就是一个能自动抓取、分类、推荐信息的全栈平台。记录一下技术选型和实现思路。

{% note info %}
项目地址在 GitHub 组织 Spoton-team 下，分为前端、后端、爬虫三个独立仓库。
{% endnote %}

## 系统架构

整体分三层，数据流方向很清晰：

{% mermaid %}
graph LR
  A[Python 爬虫] -->|写入| B[MySQL]
  B -->|读取| C[Go 后端]
  C -->|REST API| D[Vue3 前端]
  E[Redis] -->|缓存| C
  C -->|调用| F[Dify AI分类]
{% endmermaid %}

爬虫负责从学术网站、竞赛平台、技术博客抓取内容，存到 MySQL。后端从数据库读数据，通过 API 返回给前端展示。Redis 做缓存加速，Dify 做内容智能分类。

## 后端 — Go + Gin

后端用的 Go 1.21 + Gin 框架，经典的三层架构：

{% folding blue, 项目结构 %}
```
backend/
├── cmd/server/main.go     # 入口
├── configs/config.yaml    # 配置文件
├── internal/
│   ├── handler/           # HTTP 处理器
│   ├── service/           # 业务逻辑
│   ├── repository/        # 数据访问
│   ├── model/             # 领域模型
│   ├── router/            # 路由注册
│   ├── client/            # 外部服务客户端
│   └── scheduler/         # 定时任务
└── migrations/mysql/      # 数据库迁移
```
{% endfolding %}

技术栈：

- {% span blue, Gin %} — HTTP 路由和中间件
- {% span blue, go-redis %} — Redis 缓存层
- {% span blue, go-sql-driver/mysql %} — MySQL 数据库驱动
- {% span blue, robfig/cron %} — 定时任务（每天凌晨4点跑推荐，2点跑关键词统计）

数据库一共 15 张表，包括用户、信息、标签、收藏、浏览历史、搜索记录、通知、反馈、爬虫源、推荐记录等。

{% note info %}
后端运行在 8081 端口，前端通过 Vite dev server 的 proxy 把 /api 请求代理过去。
{% endnote %}

## 前端 — Vue 3 + Element Plus

前端用 Vue 3 Composition API + Vite + Element Plus + Pinia，目前是脚手架阶段，页面和组件都已经搭好框架：

- {% span green, 7 个页面 %} — 首页、列表页、详情页、搜索结果、用户中心、收藏、浏览历史
- {% span green, 8 个公共组件 %} — 导航栏、搜索框、分类侧栏、信息流、筛选栏、信息卡片、通知、偏好设置弹窗
- {% span green, 5 个 Pinia Store %} — 用户、主题、搜索、收藏、历史

前端运行在 3000 端口，支持暗色模式和响应式布局。

## 爬虫 — Python + Scrapy

爬虫用 Python 3.11 + Scrapy 框架，写了三种爬虫：

{% folding green, 三种爬虫类型 %}
| 爬虫 | 目标 | 说明 |
|------|------|------|
| AcademicSpider | 学术网站 | 抓取学术论文、研究成果 |
| CompetitionSpider | 竞赛平台 | 抓取竞赛信息和公告 |
| TechBlogSpider | 技术博客 | 抓取技术文章和教程 |
{% endfolding %}

爬虫把数据写入 MySQL，同时支持回调 Go 后端的分类接口，让 Dify 自动给内容打标签。

## 本地开发环境

用 Docker Compose 管理 MySQL 和 Redis，一条命令启动基础设施：

{% folding cyan, docker-compose.yml %}
```yaml
services:
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: rootpassword
      MYSQL_DATABASE: spoton
    ports:
      - "3306:3306"

  redis:
    image: redis:7.2
    ports:
      - "6379:6379"
```
{% endfolding %}

启动顺序：MySQL + Redis → 后端 → 前端，爬虫按需手动跑。

## 踩坑记录

{% checkbox checked red, 前端代理端口写成了 8080，后端实际是 8081 %}
{% checkbox checked red, Go 模块下载要用国内代理 goproxy.cn %}
{% checkbox checked red, Redis 在 Windows 上没有官方版本，用 tporadowski 的移植版 %}

## 总结

整个项目的技术选型还是偏主流的：Go 做后端性能好、Vue3 前端生态成熟、Scrapy 爬虫框架稳定。目前核心功能框架已经搭好，接下来主要是填充前端页面的业务逻辑和优化推荐算法。

{% note primary %}
项目还在持续开发中，欢迎关注后续更新。
{% endnote %}
