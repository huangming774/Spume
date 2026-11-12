---
title: CC Switch —— 一站管理 Claude Code / Codex / Gemini CLI 的瑞士军刀
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/Snipaste_2026-05-13_17-13-24.png
date: 2026-05-13 17:30:00
categories: 教程
tags: [AI工具, Claude Code, Skills, 效率, AI编程]
description: CC Switch 是一款跨平台桌面应用，一站式管理 Claude Code、Codex、Gemini CLI、OpenCode 等 AI 编程工具，支持 50+ 供应商一键切换、MCP/Skills 统一管理、本地代理与故障转移。
---

{% note info %}
如果你同时使用 **Claude Code**、**Codex**、**Gemini CLI** 等多个 AI 编程 CLI 工具，你一定经历过这些痛苦：改配置文件切来切去、API Key 记不住、供应商地址找半天、MCP 服务器每个工具配一遍……

**CC Switch** 就是来解决这些问题的。它由 GitHub 用户 [farion1231](https://github.com/farion1231) 开发，开源免费（MIT 协议），截止 2026 年初已斩获 **5.4 万 Star**。
{% endnote %}

---

## {% span blue, 一、CC Switch 是什么？ %}

{% label CC Switch blue %} 是一款基于 **Tauri 2** 构建的跨平台桌面应用，核心定位是 **AI CLI 工具的全能配置管理助手**。

简单说：它是一个可视化的配置面板，让你不用再手动编辑 JSON / TOML / .env 文件来切换 AI 模型供应商。

{% note primary no-icon %}
支持的应用：Claude Code、Codex（OpenAI）、Gemini CLI、OpenCode、OpenClaw、Hermes Agent
{% endnote %}

---

## {% span purple, 二、核心功能一览 %}

### {% span orange, 1. 多供应商一键切换 %}

内置 {% label 50+ blue %} 预设供应商，覆盖几乎所有主流 AI API 平台：

| 分类 | 供应商 |
|------|--------|
| {% label 官方 blue %} | Anthropic、OpenAI、Google Gemini |
| {% label 国产 red %} | DeepSeek、Kimi、智谱 GLM、Moonshot、百川、Qwen |
| {% label 聚合平台 green %} | OpenRouter、SiliconFlow、AIHubMix、NVIDIA NIM |
| {% label 云服务 purple %} | AWS Bedrock、Azure、Google Cloud |

支持 **Anthropic Messages 原生格式**和 **OpenAI Chat Completions 兼容格式**，无论供应商用哪种 API 格式都能适配。

{% tip success %}
切换供应商只需在 CC Switch 中选择对应配置，所有关联的 CLI 工具即时生效，无需重启终端。
{% endtip %}

### {% span orange, 2. MCP 服务器统一管理 %}

{% checkbox checked blue, 单一面板管理所有应用的 MCP 配置 %}
{% checkbox checked green, 支持 stdio / http / sse 三种传输类型 %}
{% checkbox checked cyan, 内置预设模板（fetch、context7、sequential-thinking 等） %}
{% checkbox checked purple, 冲突检测与字段校验 %}
{% checkbox checked orange, 跨 Claude Code / Codex / Gemini CLI / OpenCode 同步 %}

不用再每个工具重复配置一遍 MCP 了，CC Switch 自动帮你做双向同步。

### {% span orange, 3. Skills 管理 %}

这是让我最惊喜的功能之一。CC Switch 可以：

{% checkbox checked blue, 自动扫描 GitHub 上的热门 Claude Skills 仓库 %}
{% checkbox checked green, 一键安装 / 卸载 / 更新 Skills，支持批量操作 %}
{% checkbox checked cyan, 支持从本地 ZIP 安装 %}
{% checkbox checked purple, 内置 baoyu-skills 等预设仓库 %}
{% checkbox checked orange, 版本管理和回滚 %}

相当于一个 Skills 的应用商店，不用手动下载解压拷贝了。

### {% span orange, 4. 本地路由 & 故障转移 %}

内置了一个高性能 HTTP 本地代理，提供：

| 能力 | 说明 |
|------|------|
| {% label 请求监控 blue %} | 实时查看 API 请求/响应 |
| {% label 自动故障转移 red %} | 主供应商挂掉自动切备用 |
| {% label 熔断器 green %} | 连续失败达阈值自动暂停 |
| {% label 健康监控 purple %} | 供应商延迟可视化面板 |
| {% label 用量统计 orange %} | Token 消耗、成本追踪、缓存命中率 |

还有一个{% span red, Claude Rectifier %}功能——修复第三方 API 网关的 thinking block 签名格式问题，让 DeepSeek 等国产模型在 Claude Code 中也能完整输出思考过程。

### {% span orange, 5. 更多实用功能 %}

| 功能 | 简述 |
|------|------|
| {% label 系统托盘 blue %} | 托盘秒切供应商，不用打开主窗口 |
| {% label 会话管理 green %} | 浏览、搜索、恢复、删除跨应用对话历史 |
| {% label 云同步 purple %} | WebDAV / Dropbox / OneDrive / iCloud 多设备同步配置 |
| {% label 深链接 orange %} | `ccswitch://` 协议一键导入供应商/MCP 配置 |
| {% label Prompts 管理 cyan %} | Markdown 实时编辑预览，跨应用同步 CLAUDE.md |
| {% label 三语界面 blue %} | 中文 / English / 日本語 |

---

## {% span green, 三、安装方式 %}

{% folding blue, Windows %}
前往 [Releases 页面](https://github.com/farion1231/cc-switch/releases) 下载最新 **MSI 安装包**或 Portable 免安装版。

双击安装即可，支持 Windows 10+。
{% endfolding %}

{% folding green, macOS %}
推荐 Homebrew 安装：

```bash
brew tap farion1231/ccswitch
brew install --cask cc-switch
```

也支持从 Releases 页面下载 DMG / ZIP。
{% endfolding %}

{% folding cyan, Linux %}
**Arch Linux (AUR)：**
```bash
paru -S cc-switch-bin
```

**Debian/Ubuntu：**
```bash
wget https://github.com/farion1231/cc-switch/releases/latest/download/CC-Switch-latest-Linux.deb
sudo dpkg -i CC-Switch-latest-Linux.deb
```

也支持 rpm / AppImage / Flatpak。
{% endfolding %}

---

## {% span cyan, 四、实际使用体验 %}

我把 CC Switch 用在日常开发中一段时间，最大的感受是：

### 配置效率

以前换一个供应商要改 Claude Code 的 settings.json，再改 Codex 的 TOML，还要改 .env……现在在 CC Switch 里选一下，所有工具自动生效。

### MCP 不再重复配置

MCP 服务器在 CC Switch 里配置一次，Claude Code 和 Codex 都能用，新增工具时也只需要加一次。

### Skills 安装超方便

之前装 Skills 要手动 clone 或者下载 ZIP 解压到 .claude/skills/ 目录，现在在 CC Switch 里浏览 → 点击安装 → 搞定。

### 故障转移保住效率

用 DeepSeek 的时候偶尔会遇到 API 限流，本地路由自动切到备用供应商，基本无感切换。

---

## {% span orange, 五、技术架构 %}

CC Switch 的技术选型也值得一看：

{% note default no-icon %}
**前端：** React 18 + TypeScript + Vite + TailwindCSS + shadcn/ui + TanStack Query v5
**后端：** Rust（Tauri 2）
**数据存储：** SQLite
**跨平台：** Windows / macOS / Linux（x64 + arm64）
{% endnote %}

Tauri 2 让它体积小巧（相比 Electron 应用），同时 Rust 保证了本地代理的性能。

---

## {% span red, 六、总结 %}

{% progress 100 blue 推荐指数：满分 %}

{% note success no-icon %}
如果你每天和 Claude Code / Codex / Gemini CLI 打交道，CC Switch 是必装工具。它把"改配置"这件事从 5 分钟压缩到 5 秒，加上 MCP 和 Skills 管理、本地路由这些锦上添花的功能，完全可以取代手动管理配置文件的原始方式。
{% endnote %}

项目地址：[https://github.com/farion1231/cc-switch](https://github.com/farion1231/cc-switch)

---

## {% span blue, 参考资料 %}

- [CC Switch GitHub 仓库](https://github.com/farion1231/cc-switch)
- [CC Switch 用户手册](https://docs.packyapi.com/docs/ccswitch/)
- [Ruanyf Weekly 开源自荐](https://github.com/ruanyf/weekly/issues/8674)

---

*Happy coding with cc-switch!*
