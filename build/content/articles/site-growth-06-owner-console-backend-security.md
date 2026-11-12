---
date: 2026-06-07
slug: site-growth-06-owner-console-backend-security
title_zh: 建站成长记录 06｜站长控制台后端接入与安全边界
title_en: Site Growth Log 06 | Owner Console Backend Integration and Security Boundaries
excerpt_zh: 这篇记录站长控制台从原型走向真实后端的一次更新：收件箱、文章发布、友链发布、图片上传、相册发布和注册用户查看都接入了 owner-only 接口，同时也梳理了公开仓库里哪些信息可以被看到、哪些秘密必须留在环境变量和 GitHub Secrets 里。
excerpt_en: This post records the owner console moving from prototype behavior to real backend workflows: inbox handling, article publishing, friend-link publishing, asset uploads, gallery publishing, and registered-user review, while clarifying which information is visible in the public repository and which secrets stay in environment variables and GitHub Secrets.
words: 4300
reads: 42
minutes: 17
---

# 建站成长记录 06：站长控制台真正接入后端之后，安全边界也要说清楚

这次更新不是单纯再加一个页面，也不是把按钮文案改得更像控制台。

它更像是把站长控制台从“看起来能操作”的原型，推进到“真的能写入后端和仓库”的阶段。

以前有些地方只是流程预览：

- 留言收件箱能看到有提醒，但看不到具体内容
- 标记已读按钮存在，但没有真实动作
- 友链发布按钮更像预览，不会真正写入数据
- AI 注册用户只能看到一部分统计，而不是完整账号列表
- 发文章和图片链路虽然有雏形，但还需要端到端确认

这轮之后，站长控制台开始承担更真实的后台职责。

它不再只是一个漂亮的管理界面，而是一个带权限、带写入、带发布动作的 owner-only 工具。

## 这次新增和接通了什么

这轮后端最重要的变化，是把几个站长操作都收回到 `/api/owner/` 这一组接口下面。

这样做的好处很直接：

- 只有站长 session 能访问
- 前端不用直接碰 GitHub Token 或 COS 密钥
- 写入行为统一经过后端校验
- 控制台能显示真实结果，而不是假成功

现在控制台里几个关键动作已经变成真实链路。

| 功能 | 当前状态 | 后端作用 |
| --- | --- | --- |
| 站长收件箱 | 显示逐条留言内容 | 从留言表读取未读消息 |
| 标记已读 | 真实写入已读状态 | 只写 `owner_read_at`，不隐藏公开留言 |
| AI 注册用户 | 显示完整非站长用户列表 | 返回邮箱、昵称和注册时间 |
| 发布文章 | 写入博客 Markdown | 通过后端提交到 GitHub 内容接口 |
| 发布友链 | 写入友链数据文件 | 避免前端直接改仓库 |
| 上传图片到 COS | 由后端上传资源 | 浏览器只提交文件，不接触 COS 密钥 |
| 发布到相册 | 写入相册数据文件 | 支持重复图片检测 |

这几个功能组合起来之后，控制台的性质就变了。

它不再只是展示状态，而是开始真正管理站点内容。

## 收件箱为什么要单独做已读字段

这次留言收件箱里最需要小心的点，是“标记已读”的语义。

一开始最容易想到的做法，是点已读之后把留言隐藏。

但这其实不对。

因为“站长已经读过”和“公开留言是否展示”是两件事。

如果把已读直接做成隐藏，站长每点一次已读，就会把访客的公开留言从网站上撤下来。这样虽然收件箱看起来干净了，但公开页面会被误伤。

所以这次选择了更稳的方式：

> 给留言记录增加 `owner_read_at`，站长收件箱只筛选未读留言；公开留言状态仍然保持不变。

也就是说：

- 访客留言仍然可以公开展示
- 站长收件箱可以只看未处理消息
- 点已读只影响站长视角
- 不会因为整理收件箱而改变公开内容

这个边界很小，但很重要。

后台操作一旦接入真实写入，就不能只考虑“按钮点了有没有反应”，还要考虑这个动作会不会误伤公开页面。

## 友链和文章为什么要让后端发布

文章和友链本质上都会改仓库里的内容文件。

如果让前端直接完成这件事，就会遇到一个根本问题：

> 浏览器不能拿到发布用的 GitHub Token。

一旦 Token 出现在前端代码、网页源码、打包后的 JS 或浏览器请求里，它就等于公开了。

所以现在的方式是：

1. 前端只提交标题、正文、友链信息或图片 URL
2. 后端检查当前 session 是否真的是站长
3. 后端读取目标数据文件
4. 后端生成更新内容
5. 后端用部署环境里的发布凭据提交到仓库
6. 前端只拿到发布结果、路径和 commit 信息

这个结构的重点不是“让流程复杂一点”，而是把秘密留在服务器侧。

前端可以知道有一个“发布文章”的接口，也可以知道接口路径。

但前端不能知道发布凭据本身。

## 图片上传为什么也要走后端

图片上传到 COS 也是同一个道理。

COS 的 Secret ID 和 Secret Key 不能放在前端。

浏览器只应该做一件事：

> 把站长选择的图片文件提交给后端。

真正的 COS 上传由后端完成。

这样即使别人打开开发者工具，也只能看到：

- 前端调用了上传接口
- 请求需要站长登录态
- 上传结果返回一个公开图片 URL

他们不应该看到：

- COS Secret Key
- COS 写入权限
- GitHub 发布 Token
- 站长登录密码
- 二次验证答案

这也是为什么“接口路径公开”并不等于“权限公开”。

一个长期运行的网站不可能把所有 API 路径都藏起来。真正需要保护的是鉴权、权限判断、密钥和写入能力。

## 别人拉到 GitHub 仓库后能看到什么

这次我也重新检查了仓库里会公开的信息。

如果别人 clone 这个项目，他可以看到这些内容：

| 会看到 | 是否敏感 | 说明 |
| --- | --- | --- |
| 后端代码结构 | 正常公开 | 开源仓库本来就会暴露实现方式 |
| API 路径 | 正常公开 | 前端 JS 和 README 里也会出现接口路径 |
| 部署 workflow | 正常公开 | 可以看到部署步骤和 Secrets 名称 |
| `.env.example` | 正常公开 | 只应该放占位符 |
| 站长保留账号的邮箱配置 | 中等敏感 | 不是密码，但会暴露 owner 账号标识 |
| 二次验证的问题文本 | 中等敏感 | 能看到问题，但不应该看到答案 |

这些内容里，最需要理解的是 API 路径。

别人知道 `/api/owner/status`、`/api/owner/publish` 或 `/api/owner/assets` 这种路径，并不代表他能调用成功。

因为这些接口真正依赖的是：

- 有效登录 session
- 用户必须是站长
- 站长 session 必须通过二次验证
- 后端必须从服务器环境变量里拿到发布或上传凭据

没有这些条件，接口应该返回未登录或无权限。

所以 API 路径可以被看见，权限不能被绕过。

## 别人不应该看到什么

公开仓库里不应该出现下面这些真实值：

- 站长登录密码
- 二次验证答案
- DeepSeek API Key
- GitHub 发布 Token
- COS Secret Key
- UCloud SSH 私钥
- UCloud sudo 密码
- 服务器 `/opt/acg-api/.env` 的真实内容
- 数据库里的用户 session

这次检查的结果是：

- 仓库里 tracked 的 `.env` 文件没有被提交
- `.gitignore` 明确忽略了 `acg-api/.env`
- Git 历史里没有发现 `.env` 被提交过
- workflow 里出现的是 GitHub Secrets 的变量名，不是真实值
- `.env.example` 里是占位符，不是真实密钥

这说明当前仓库的基本安全边界是对的。

但也有两个公开信息需要心里有数：

1. owner 邮箱是公开代码里能看到的
2. 二次验证的问题文本是公开代码里能看到的

这两项不是密码，也不是答案。

但它们确实会给别人提供“该尝试哪个账号”和“二次验证问什么”的信息。

如果以后要继续加固，可以把 owner 邮箱和问题文本也挪到环境变量里。这样仓库只保留默认值或占位配置，真实 owner 账号由服务器决定。

## 上传后端会不会把秘密也上传出去

这里要分清两种“上传”。

第一种是上传后端源码到 GitHub。

如果上传源码，别人会看到：

- 后端接口怎么写
- 数据库表大概怎么设计
- 权限判断逻辑
- 环境变量名字
- 部署脚本流程

但只要没有把 `.env`、真实 Token、真实密码和私钥提交进去，别人就不会从源码里拿到你的站长密码和密钥。

第二种是把后端部署到服务器。

部署时 GitHub Actions 会构建 Go 二进制，把二进制、service 文件和部署脚本传到服务器。

真正的秘密值来自 GitHub Secrets，并同步进服务器的 `/opt/acg-api/.env`。

这个文件应该只存在服务器上，并且权限是 `600`。

所以正常情况下：

> 上传后端二进制本身不会暴露密码；风险主要来自把 `.env` 误提交、把 Secrets 打印到日志、服务器权限过宽，或者把私钥/Token 写进源码。

当前这套部署方式的方向是正确的：源码和构建产物公开，秘密留在 GitHub Secrets 和服务器环境文件。

## 什么是可以公开的，什么必须保密

我现在更愿意把安全边界理解成这张表。

| 类型 | 可以公开吗 | 例子 |
| --- | --- | --- |
| API 路径 | 可以 | `/api/chat`、`/api/guestbook/messages` |
| 接口用途 | 可以 | 登录、留言、发布、上传 |
| 环境变量名 | 可以 | 用来说明需要配置什么 |
| 示例配置 | 可以 | `your_secret_here` 这种占位值 |
| 真实密码 | 不可以 | 站长登录密码 |
| 真实验证答案 | 不可以 | 二次验证答案 |
| 真实 Token | 不可以 | GitHub 发布 Token |
| 云服务 Secret | 不可以 | COS Secret Key |
| session cookie | 不可以 | 用户登录态 |
| 数据库文件 | 不应该公开 | 用户、留言、session 数据 |

所以别人拉到项目后，知道“这个项目有 owner API”是正常的。

但他不能知道“怎么成为 owner”。

这就是区别。

## 这次更新后的阶段感

到这一步，站长控制台已经不再是一个孤立页面。

它开始和整个网站的内容系统连在一起：

- 留言来自真实后端
- 已读状态写回数据库
- 文章可以发布到博客源码
- 友链可以写入主站数据
- 图片可以上传到 COS
- 相册可以更新站内数据
- AI 注册用户可以被站长完整查看

这意味着网站越来越像一个可以长期运行的小型个人应用。

也正因为它开始真实写入，安全边界必须跟着变清楚。

我的基本原则是：

> 能公开的是结构，不能公开的是凭据；能暴露的是接口形状，不能暴露的是通过接口获得写入权限的秘密。

只要这个原则守住，别人看见项目代码并不可怕。

真正危险的是把密码、答案、Token、私钥或服务器环境文件一起交出去。

这也是这次更新最重要的收尾：功能接通之后，要立刻确认它没有把不该公开的东西带到公开仓库里。

---en---

# Site Growth Log 06: After the Owner Console Reaches the Real Backend, the Security Boundary Has to Be Clear

This update is not just another UI polish pass.

It moves the owner console from a prototype-like control surface into a real backend-connected tool.

Several actions now go through owner-only backend APIs:

- unread guestbook inbox items can show actual message content
- marking a message as read writes an owner-only read timestamp
- article publishing goes through the backend
- friend-link publishing writes the site data through the backend
- image upload is handled by the backend instead of exposing cloud secrets to the browser
- gallery publishing updates the gallery data file
- registered users can be reviewed from the owner console

The key design rule is simple:

> The frontend may know the API shape, but it must never know the credentials.

API paths are not secrets. A public site cannot realistically hide every route from users, because the browser has to call many of them. The real protection must come from authentication, authorization, server-side validation, and keeping credentials out of frontend code.

## What a public repository can reveal

If someone clones the repository, they can see:

- backend source code
- public API paths
- deployment workflow files
- environment variable names
- placeholder `.env.example` values
- the general owner verification flow

That is expected for a public codebase.

What they must not see is:

- the owner login password
- the secondary verification answer
- real API keys
- GitHub publishing tokens
- cloud storage secrets
- SSH private keys
- production `.env` content
- session cookies
- database files containing live user data

The repository scan for this update confirmed the important baseline:

- the tracked repository does not include the real `.env`
- `acg-api/.env` is ignored
- the Git history does not show the real `.env` being committed
- workflow files reference GitHub Secrets by name, not by value
- `.env.example` contains placeholders, not real credentials

There are still two pieces of information that are visible in the codebase: the owner account identity and the secondary question text. They are not passwords or answers, but they are still more identifying than ordinary configuration. If the project needs stronger privacy later, they can also be moved into server-side environment variables.

## Does deploying the backend expose secrets?

Deploying the backend has to be separated into two cases.

Uploading source code to GitHub reveals the implementation and route structure, but it should not reveal secrets as long as real environment files and credentials stay out of Git.

Deploying the backend binary to the server also does not embed the owner password or cloud secrets by itself. The service reads those values from the server environment at runtime.

So the main risks are not “the backend exists” or “the API path is visible.” The real risks are:

- accidentally committing `.env`
- printing secrets into CI logs
- storing credentials in frontend code
- giving the server environment file loose permissions
- putting tokens directly into source files

The safer pattern is the one used here:

1. keep source code and build artifacts public
2. keep real credentials in GitHub Secrets and server-side environment files
3. require owner sessions for write actions
4. let the backend perform publishing and upload operations
5. return only safe result metadata to the browser

This is the core boundary of the current system:

> The structure can be public. The authority must stay private.

