---
title: Reimu 主题博客搭建笔记——Hexo 新主题从零到部署
date: 2026-05-10 21:00:00
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/Snipaste_2026-05-10_20-49-32.png
categories: 教程
tags: [Hexo, blog, Skills]
description: 从 Butterfly 迁移到 Reimu 主题的完整踩坑记录
---

# Reimu 主题博客搭建笔记——Hexo 新主题从零到部署

{% note info %}
本文记录了从零搭建一个全新 Hexo 博客的完整过程——使用 {% label 拔剑Sketon blue %} 开发的 **博丽灵梦风格 Reimu 主题**，从初始化项目、配置主题、文章迁移、标签适配，到 Vercel 部署上线的全部踩坑记录。如果你也想换个新主题，这篇文章应该能帮你省不少时间。
{% endnote %}

{% link 拔剑Sketon 的博客, https://d-sketon.github.io/, https://d-sketon.github.io/avatar/avatar.webp %}

---

## 为什么换主题？

Butterfly 是一套非常成熟的 Hexo 主题，功能齐全。但在浏览 {% span red, D-Sketon（拔剑Sketon）%}的博客时，被 Reimu 主题的**简洁东方风设计**吸引：

{% checkbox checked blue, 灵梦主题配色 + 太极图标动画 %}
{% checkbox checked green, AOS 滚动动画 + PJAX 无刷新加载 %}
{% checkbox checked purple, 内置照片墙、标签页、友链卡片、热力图 %}
{% checkbox checked orange, 暗色模式、响应式布局、多语言支持 %}
{% checkbox checked red, 比 Butterfly 更轻量，风格更现代 %}

---

## 搭建过程

### 1. 初始化项目

在 D 盘新建 `blog1` 目录：

```bash
cd D:/blog1
npx hexo init .
npm install hexo-theme-reimu
```

`_config.yml` 中设置 `theme: reimu`，同时配置站点信息：

```yaml
title: 桃之夭夭の花园
author: 桃之夭夭
language: zh-CN
```

{% note info %}
Reimu 的配置文件是 `_config.reimu.yml`（不是 Butterfly 的 `_config.butterfly.yml`）。配置项包括导航菜单、打字机副标题、社交链接、侧边栏、i18n 等。
{% endnote %}

### 2. 核心配置

从原博客复制了头像、社交链接等信息，重点配置了以下部分：

**打字机副标题：**

```yaml
subtitle:
  typing:
    enable: true
    strings:
      - 学无止境，持之以恒
      - 千里之行，始于足下
    loop: true
```

**社交链接（GitHub + B站 + 邮箱）：**

```yaml
social:
  github: https://github.com/bistutzyy
  bilibili: https://space.bilibili.com/1061280173
  email: mailto:173236231@qq.com
```

**侧边栏：** 只保留个人信息和标签云，去掉了分类、归档、最新文章：

```yaml
widgets:
  - tagcloud
```

**导航菜单：**

```yaml
menu:
  - name: home
    url: /
  - name: archives
    url: /archives
  - name: about
    url: /about
  - name: friend
    url: /friend
```

### 3. 文章迁移

从原博客复制了 14 篇文章。但这里遇到了 {% span red, 第一个大问题 %}——Butterfly 标签和 Reimu 标签互不兼容。

### 4. 关于页与友链页

**关于页** 写入了个人介绍和 GitHub 贡献表：

- 个人简介 + 博客介绍
- GitHub 贡献热力图（使用 `ghchart.rshah.org` 服务，从 GitHub 实时拉取提交数据生成 SVG）
- 建站日期、技术栈

**友链页** 加入原博客 bistutzyy.github.io 作为友链，命名为「桃之夭夭第一章」。

### 5. 部署到 Vercel

在 GitHub 新建仓库 `bistutzyy/blog1-reimu`，推送代码。然后在 Vercel 导入该仓库。

{% note danger %}
Vercel 默认自动识别 Hexo 项目，构建命令为 `hexo generate`，输出目录为 `public`。但需要注意：
- 需要创建 `vercel.json` 配置文件
- 本地 Windows 的 `package-lock.json` 在 Vercel Linux 环境可能导致依赖版本冲突，建议 `.gitignore` 忽略它
- Vercel 要求 Node 24.x，需在 `package.json` 中设置 `"engines": { "node": "24.x" }`
{% endnote %}

---

## 遇到的问题与解决

{% timeline 踩坑记录, red %}

<!-- timeline 问题一：Butterfly 标签完全不兼容 -->
**现象：** 复制过来的文章大量使用 note 标签、span 标签、folding 标签 等 Butterfly 标签，生成时报 `unknown block tag` 错误。

**原因：** Reimu 有自己的标签体系（`alertBlockquote`、`details`、`gallery`、`tabs` 等），和 Butterfly 完全不兼容。

**解决：** 批量替换标签：

| Butterfly | Reimu |
|-----------|-------|
| note | alertBlockquote |
| folding | details |
| span/p | `<span class="c-color">` |
| label | badge（需自定义） |
| tabs | tabs（Reimu 原生，语法相同） |
| gallery | gallery（需重写，见问题二） |

对于 Reimu 没有的标签，在项目的 `scripts/` 目录下创建自定义标签脚本。所有自定义标签最后合并到 `scripts/tags.js`。

<!-- endtimeline -->
<!-- timeline 问题二：照片墙 JS 代码泄漏到页面 -->
**现象：** Vercel 部署后，文章标题下方出现大段 JavaScript 代码文本。

**原因：** Reimu 原版 gallery 标签会生成内联 `<script>` 实现 photoWall 效果。在 Vercel 构建环境中，这些内联 JS 被错误暴露为可见文本。

**解决：** 将 gallery 标签重写为纯 CSS 瀑布流：

```javascript
// scripts/gallery-override.js
hexo.extend.tag.register("gallery", function (args, content) {
  const html = hexo.render.renderSync({ text: content, engine: "markdown" });
  // 提取所有 img 标签的 src
  const imgRegex = /<img[^>]+src=["']([^"']+)["'][^>]*>/g;
  let match, items = [];
  while ((match = imgRegex.exec(html)) !== null) {
    items.push({ src: match[1] });
  }
  // 生成纯 CSS 瀑布流 HTML
  let imgs = items.map(item =>
    '<div class="gm-item"><a href="' + item.src + '">' +
    '<img src="' + item.src + '" loading="lazy"></a></div>'
  ).join('');
  return '<div class="gm-gallery">' + imgs + '</div>';
}, { ends: true });
```

配套 CSS 使用 `columns` 实现真瀑布流，每张图保持原始比例，不裁切。


<!-- endtimeline -->
<!-- timeline 问题三：轮播图在 Reimu 中完全无法使用 -->
**现象：** Butterfly 的 `carousel` 标签依赖 `carousel-touch.js` 和 `transform-style: preserve-3d`，与 Reimu 的 CSS 布局冲突，3D 旋转失效。

**尝试过程：**
- 尝试一：保留 Butterfly carousel-touch.js，用 CSS 修复 3D 变换 → {% span red, 图片大小不一，旋转方向错乱 %}
- 尝试二：改用纯 CSS `scroll-snap` 滑动画廊 → {% span red, 图片过大，毫无新意 %}
- 尝试三：写自定义 CSS 展示型轮播，淡入淡出切换 → {% span red, 竖屏图片显示不全，人脸被裁 %}

**最终方案：** 放弃轮播图，改用 gallery 瀑布流直接展示所有图片。

{% note info %}
轮播图是 Butterfly 和 Reimu 之间最深层的架构冲突——Butterfly 的 3D carousel 依赖特定的 CSS transform 环境和外部 JS，而 Reimu 的布局完全不兼容。与其勉强修复，不如使用主题原生的 gallery。
{% endnote %}

<!-- endtimeline -->
<!-- timeline 问题四：中英双语切换始终无法生效 -->
**现象：** 配置 `language: [zh-CN, en]` 并在 `source/en/_posts/` 下放置英文翻译，切换语言后文章正文仍是中文。

**原因（核心）：** 经过反复测试（Hexo 7.3.0、8.1.2、安装 `hexo-generator-i18n` 插件），确认 **Hexo 不会自动扫描 `source/en/_posts/` 目录**。`source/en/` 只对页面（about、friend）生效，Post 不受支持。

**尝试的三种方案：**

| 方案 | 原理 | 结果 |
|------|------|------|
| `en-posts-loader.js` | 在 `before_generate` 阶段手动将 `source/en/_posts/` 下的英文文章注入数据库 | 本地有效，Vercel 不生效 |
| slug 级切换 | 英文文章以 `en-` 前缀命名放在 `source/_posts/`，修改 `switch_lang` 函数实现路径映射 | 本地有效，Vercel build 脚本不稳定 |
| 放弃 | 关闭 i18n，博客保持纯中文 | ✅ 稳定可靠 |

**为什么 Vercel 上不生效？**

`en-posts-loader.js` 需要在 Hexo 的 `before_generate` 钩子中通过 `Post.insert()` 注入文章。Vercel 的构建环境（Linux + Node 24）下，`hexo-front-matter` 的 `parse()` 对含有特殊字符（如中文冒号）的 YAML 解析失败，导致英文文章静默跳过。而补丁脚本修改 `node_modules/` 的方式在 Vercel 上不可靠——`npm install` 会还原所有文件。

{% note danger %}
这是本次搭建中耗时最长（超过 3 小时）、尝试方案最多（4 种）、最终也未能完美解决的问题。核心原因是 Hexo 框架本身不提供文章级多语言目录扫描，而 Vercel 又限制了运行时对 `node_modules` 的修改。D-Sketon 作为 Hexo 核心贡献者，他博客的中英切换可能是通过定制 Hexo 实现的。
{% endnote %}

<!-- endtimeline -->
<!-- timeline 问题五：Vercel 构建反复失败 -->
**现象：** 推送代码后 Vercel 构建持续报错 `Nunjucks Error: unknown block tag`，且每次错误都不同（badge → endtimeline → endtabs → poem）。

**原因链：**
1. 项目 `scripts/` 目录在清理过程中被误删，导致自定义标签（badge、mermaid、timeline）丢失
2. `hexo-butterfly-tag-plugins-plus` 在 Node 24 + Vercel 环境存在兼容问题
3. 文章中的 标签语法示例被 Hexo 的 Nunjucks 解析器当作真实标签处理

**解决步骤：**
1. 将所有自定义标签合并到 `scripts/tags.js` 中统一管理
2. 保留 `hexo-butterfly-tag-plugins-plus` 依赖（poem、video等标签需要）
3. 文章中涉及标签语法的部分全部用文字描述替代（如"【note 标签】"），避免被 Nunjucks 误解析
4. 在 `package.json` 中设置 `engines.node: "24.x"`
5. 创建 `vercel.json` 指定构建命令和输出目录

{% note success %}
这个问题的根本教训是：**绝对不要在 Hexo 文章正文中直接写 tag 语法**。Hexo 的 Nunjucks 解析器在 Markdown 渲染前运行，代码块和行内代码都挡不住它。
{% endnote %}

<!-- endtimeline -->
<!-- timeline 问题六：封面图与轮播图的后续处理 -->
**封面图问题：** Reimu 的 `post.ejs` 会自动给没有 `cover` 字段的文章分配随机封面或使用主题 banner，导致首页卡片都有封面图。

**解决：** 修改 `layout/_partial/post.ejs`，将封面图逻辑改为仅当文章显式设置了 `cover` 字段时才显示。这需要在 Vercel build 前通过补丁脚本修改 `node_modules/` 中的文件……最终用 CSS 暴力方案：`.post-cover { display: none }`

**文章顶部 banner：** 设置 `cover:` 为空值（不是 `false`），这样文章页面顶部会使用主题默认的 banner 图，而首页卡片无封面。

**轮播图：** 用 gallery 瀑布流替代，彻底移除了 Butterfly carousel 的所有依赖（JS、CSS）。
{% endtimeline %}

---

## 自定义标签实现

Reimu 缺少的标签，在项目的 `scripts/tags.js` 中自定义：

{% note info %}
**关键规则：** Hexo 会优先加载项目 `scripts/` 目录下的脚本，加载顺序晚于主题脚本。因此项目级标签可以覆盖同名主题标签。
{% endnote %}

**badge 标签（替代 Butterfly 的 label）：**

```javascript
hexo.extend.tag.register('badge', function(args) {
  let [text, color = 'blue'] = args
  return '<mark class="rm-badge ' + color + '">' + text + '</mark>'
})
```

**timeline 标签（自定义实现）：**

```javascript
hexo.extend.tag.register('timeline', function(args, content) {
  let [title = '', color = ''] = args.join(' ').split(',')
  // 解析 <!-- timeline --> 标记，生成时间轴 HTML
  // ...
}, { ends: true })
```

**blockquote 标签（带作者署名）：**

```javascript
hexo.extend.tag.register('blockquote', function(args, content) {
  const author = args[0] ? '<p class="rm-bq-author">—— ' + args[0] + '</p>' : ''
  return '<blockquote class="rm-blockquote">' + content + author + '</blockquote>'
}, { ends: true })
```

配套 CSS 在 `source/css/tag-styles.css` 中定义，并通过 reimu 的 `injector.head_begin` 注入。

---

## 标签体系对比

{% tabs 标签对比 %}
<!-- tab Butterfly -->
| 功能 | 标签 |
|------|------|
| 便签 | note |
| 折叠 | folding |
| 彩色文字 | span / p |
| 徽章 | label |
| 时间轴 | timeline |
| 进度条 | progress |
| 轮播图 | carousel |
| 诗词 | poem |
<!-- endtab -->
<!-- tab Reimu -->
| 功能 | 标签 |
|------|------|
| 提示框 | alertBlockquote |
| 折叠 | details |
| 彩色文字 | `<span class="c-color">` |
| 徽章 | badge（自定义） |
| 时间轴 | timeline（自定义） |
| 进度条 | 手动 HTML |
| 轮播图 | gallery 替代 |
| 诗词 | poem（依赖 tag-plugins-plus） |
<!-- endtab -->
{% endtabs %}

---

## 最终成果

{% note success %}
经过一整天的折腾，Reimu 主题博客成功上线 🎉

- **地址：** [blog1-reimu.vercel.app](https://blog1-reimu.vercel.app)
- **主题：** hexo-theme-reimu v1.12.4
- **部署：** Vercel（GitHub 推送自动构建）
- **文章：** 14 篇（从原博客完整迁移）
- **标签体系：** 混合使用 Reimu 原生 + 自定义 + tag-plugins-plus
- **侧边栏：** 个人信息 + 标签云
- **友链：** 链接到原 Butterfly 博客
{% endnote %}

## 与原博客的关系

原 Butterfly 博客 `https://bistutzyy.github.io/` 保持不变，Reimu 博客独立部署在 Vercel，两者互不影响。

---

## 技术总结

{% checkbox checked blue, Hexo 7.3.0 + hexo-theme-reimu 1.12.4 %}
{% checkbox checked green, 自定义标签放在项目 scripts/ 目录（不在 node_modules/ 里改） %}
{% checkbox checked purple, node_modules 修改不持久，Vercel 部署时会 npm install 覆盖 %}
{% checkbox checked red, 文章正文绝对不能直接写标签语法（如 {% note %} 等），Nunjucks 会误解析 %}
{% checkbox checked orange, 优先使用 Reimu 原生标签，其次自定义，最后才依赖 tag-plugins-plus %}
{% checkbox checked blue, Vercel 需要 Node 24.x + vercel.json 配置 %}
{% checkbox checked green, gallery 用纯 CSS 替代 photoWall，避免内联 JS 泄漏 %}
{% checkbox checked purple, i18n 文章级切换受限于 Hexo 框架，页面级切换可用 %}

---

## 致谢与参考

{% note primary %}
感谢 {% span blue, D-Sketon（拔剑Sketon）%} 开发的 Reimu 主题！

**相关链接：**
- {% label 主题仓库 blue %}：[D-Sketon/hexo-theme-reimu](https://github.com/D-Sketon/hexo-theme-reimu)
- {% label 作者博客 purple %}：[d-sketon.github.io](https://d-sketon.github.io/)
- {% label 使用指南 green %}：[hexo-theme-reimu 食用指南](https://d-sketon.github.io/20230707/hexo-theme-reimu-guide/)
- {% label 开发日志 orange %}：[hexo-theme-reimu 开发日志](https://d-sketon.github.io/reimu-lighthouse/20240601/hexo-theme-reimu-log/)
- {% label 博客仓库 red %}：[bistutzyy/blog1-reimu](https://github.com/bistutzyy/blog1-reimu)
{% endnote %}

{% note info %}
如果你也想尝试 Reimu 主题，建议先阅读使用指南，了解主题内置的标签体系。从 Butterfly 迁移的注意点上面已经列出来了——标签替换是最关键的一步。如果有任何问题，欢迎在评论区交流！
{% endnote %}
