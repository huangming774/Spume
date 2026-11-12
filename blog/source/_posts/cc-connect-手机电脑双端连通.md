---
title: Claude Code + cc-connect 打通微信，手机远程操控电脑写代码
date: 2026-05-14 14:30:00
tags: [Claude Code, cc-connect, 微信, 效率工具, 远程控制]
categories: [教程]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/Snipaste_2026-05-14_14-42-50.png
---

{% span red, 用手机指挥电脑上的 Claude Code 写代码、发文件、改配置，全程无需碰键盘。 %}这篇教程记录我如何通过 cc-connect 把 Claude Code 接入微信，实现双端联动。

{% tip info %}
本文所有操作均在 Windows 11 + Node.js 22 环境下完成，微信端使用个人微信（ilink 通道）。
{% endtip %}

## 一、什么是 cc-connect？

{% note primary no-icon %}
cc-connect 是一个开源桥接工具，将本地 AI 编程助手（Claude Code、Cursor、Gemini CLI 等）接入微信、飞书、Telegram 等聊天平台，让你随时随地从手机远程操控 AI Agent。

项目地址：[https://github.com/chenhg5/cc-connect](https://github.com/chenhg5/cc-connect)
{% endnote %}

{% folding blue, 支持平台一览 %}
| 平台 | 支持状态 |
|------|----------|
| 飞书/Lark | ✅ 稳定 |
| Telegram | ✅ 稳定 |
| 钉钉 | ✅ 稳定 |
| Slack | ✅ 稳定 |
| Discord | ✅ 稳定 |
| 微信个人号 | ✅ Beta |
| 企业微信 | ✅ 稳定 |
| QQ | ✅ 稳定 |
| LINE | ✅ 稳定 |
{% endfolding %}

核心能力概括：

{% checkbox checked green, 手机微信直接与 Claude Code 对话 %}
{% checkbox checked green, 支持图片 / 文件 / 语音发送到电脑 %}
{% checkbox checked green, 电脑端文件可回传到微信聊天 %}
{% checkbox checked blue, 斜杠命令操控 Agent (%}{% kbd / %}{% label mode blue %}{% kbd / %}{% label new blue %}{% kbd / %}{% label stop blue %} 等) %}
{% checkbox checked blue, 无需公网 IP，扫码即连 %}

---

## 二、安装配置

### Step 1 — 安装 Beta 版

{% note warning %}
微信个人号渠道仅在 **Beta 版** 中可用。必须用 `@beta` 标签安装！
{% endnote %}

```bash
npm install -g cc-connect@beta
```

安装完成后验证：

```bash
cc-connect --version
```

### Step 2 — 初始化配置

```bash
cc-connect init
```

这会在 `~/.cc-connect/` 下生成 `config.toml`。我当前的完整配置如下：

{% folding green, 点击展开 config.toml 完整配置 %}
```toml
# cc-connect configuration
language = "zh"
attachment_send = "on"

[display]
thinking_messages = false
tool_messages = false

[log]
level = "info"

[[projects]]
name = "default"

[projects.agent]
type = "claudecode"

[projects.agent.options]
work_dir = "C:/Users/lenovo"
mode = "default"

[[projects.platforms]]
type = "weixin"

[projects.platforms.options]
token = "YOUR_TOKEN"
base_url = "https://ilinkai.weixin.qq.com"
account_id = "YOUR_ACCOUNT"
allow_from = "*"
```
{% endfolding %}

{% label 关键配置 blue %} `attachment_send = "on"` 是文件回传的总开关，不开启则电脑无法向手机发文件。

### Step 3 — 微信扫码绑定

```bash
cc-connect weixin setup --project default
```

终端会弹出二维码 → 用微信扫码确认 → token 自动写入配置。{% span green, 无需手动填 token。 %}

---

## 三、启动与使用

```bash
cc-connect
```

启动后，打开微信向机器人发消息即可交互。我把工作目录定位在 `C:/Users/lenovo`，这样 Claude Code 的所有能力（skills、memory、项目上下文）都能在手机上调用。

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/1c108889732d721111aa7c9b76eaee4c.jpg, alt=微信端聊天效果截图一, width=360px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/1fc28c042ce32d72b0cb28b31e05120a.jpg, alt=微信端聊天效果截图二, width=360px %}

---

## 四、微信端斜杠命令

在微信对话框直接输入以下命令控制 Agent：

| 命令 | 作用 |
|------|------|
| `/new [name]` | 开启新会话 |
| `/list` | 列出所有会话 |
| `/switch <id>` | 切换会话 |
| `/current` | 查看当前会话 |
| `/history [n]` | 查看历史消息 |
| `/mode default` | 标准权限模式 |
| `/mode plan` | 规划模式 |
| `/mode yolo` | 自动执行模式 |
| `/stop` | 停止当前任务 |
| `/quiet` | 开关进度消息 |
| `/dir <path>` | 切换工作目录 |
| `/model` | 切换 AI 模型 |
| `/allow <tool>` | 预授权工具 |
| `/memory` | 管理长期记忆 |
| `/cron add` | 添加定时任务 |

更多命令输入 `/help` 即可查看完整列表。

---

## 五、文件互传

### 📱 手机 → 电脑

直接在微信发文件/图片/语音给我，cc-connect 自动保存到 `.cc-connect/attachments/` 并交给 Claude Code 处理。

### 💻 电脑 → 手机

在终端使用 `cc-connect send` 命令将文件发回微信：

```bash
# 发送文件
cc-connect send --file "C:/Users/lenovo/Desktop/report_final.docx"

# 发送图片
cc-connect send --image "C:/path/to/screenshot.png"

# 同时发送文件+图片
cc-connect send --file "report.pdf" --image "chart.png"
```

{% tip success %}
发文件用 **绝对路径** 最稳妥，避免路径解析问题。
{% endtip %}

---

## 六、语音消息

cc-connect 还支持语音互转。在 `config.toml` 中添加：

```toml
[speech]
enabled = true
provider = "groq"
language = "zh"
[speech.groq]
  api_key = "gsk_你的API密钥"
  model = "whisper-large-v3"
```

{% checkbox checked , 微信语音自动转文字发给 Claude Code %}
{% checkbox checked , Claude Code 回复可选转语音播报 %}
{% checkbox , 需要安装 ffmpeg 依赖 %}

---

## 七、总结

{% note success no-icon %}
现在我不在家也能用手机指挥电脑上的 Claude Code：
- 📝 写博客、改代码、搜文件
- 📎 电脑生成的文件一键发到手机
- 🎤 开车时用语音下指令
- ⏰ 定时任务自动运行

整套配置不超过 **30 分钟**，强烈推荐给所有用 Claude Code 的朋友。
{% endnote %}

{% folding cyan, 常见问题 FAQ %}

**Q: 微信收不到机器人消息？**

检查 cc-connect 服务是否在运行，以及 token 是否过期（重新扫码即可）。

**Q: 文件发送失败？**

确认 `attachment_send = "on"` 已设置，并使用 Beta 版 `cc-connect@beta`。

**Q: 语音转文字不工作？**

需要安装 ffmpeg 并配置 Groq API key。Groq 提供免费额度，足够日常使用。

**Q: 可以多台手机同时接入吗？**

可以。多个项目在配置中分不同的 `[[projects]]` 即可。
{% endfolding %}
