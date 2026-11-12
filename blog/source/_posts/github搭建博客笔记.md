---
title: github搭建博客笔记
date: 2025-01-03 06:00:00
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/e672bb8adba89ab5d256f3e4fe3ec72.jpg
categories: 教程
tags: [github, blog]
description: 这是一篇关于github搭建博客的基础教学
---

{% note primary %}
Welcome to my first passage!
{% endnote %}

# 任务1. 关于搭建github博客的步骤学习笔记

{% span gray, （所用设备：拯救者y9000p） %}

{% timeline 搭建流程总览, blue %}
<!-- timeline 准备工作 -->
注册 GitHub 账号
<!-- timeline STEP1 -->
Fork 博客模板仓库
<!-- timeline STEP2 -->
配置 _config.yml、评论系统、谷歌分析
<!-- timeline 完成 -->
博客上线，个性化装饰
{% endtimeline %}

## 准备工作——注册github

首先，打开 [github.com](https://github.com)，来到github的主界面（如下）

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/01.png, alt=GitHub首页, width=550px %}

点击右上角进行注册（使用邮箱注册），登录。

## STEP1 — 进行 {% span red, Fork %}（重要）

打开 [mzlogin.github.io](https://mzlogin.github.io)，进入以下页面：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/02.png, alt=博客模板主页, width=550px %}

上图即为博客的模版，往下拉：

{% gallery %}
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/03.png)
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/04.png)
{% endgallery %}

接下来开始进行 {% span red, Fork %}。

{% note info %}
往下拉页面会有 Fork 指南，下面讲述详细步骤。
{% endnote %}

点击 Fork 选项，然后 **Create a new fork**。*由于个人账号已经完成，以下部分图片来自王鑫学长教学截取。*

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/05.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/06.png" style="width: 48%;">
</div>

{% span gray, Description 描述自己博客，由自己撰写，可选可不选。 %}

进行设置 Settings：

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/07.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/08.png" style="width: 48%;">
</div>

Fork 后在浏览器网址搜索自己的网站（如上图：`ZenQG.github.io`），进入以下界面，加载不出来需等几分钟。

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/09.png, alt=博客首次加载成功, width=550px %}

{% note success %}
当出现这个界面，证明前面的步骤已经成功，{% label 第一步完成 green %}！
{% endnote %}

## STEP2——处理细节，装饰博客

### 第一步：删除 CNAME 文件

此文件与域名有关，有需要可自行修改。

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/10.png, alt=找到CNAME文件, width=500px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/11.png, alt=进入CNAME文件, width=500px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/12.png, alt=删除CNAME文件步骤1, width=500px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/13.png, alt=删除CNAME文件步骤2, width=500px %}

### 第二步——修改配置，装饰自己博客

首先，打开 `_config.yml`，进入编辑：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/14.png, alt=打开_config.yml, width=500px %}

然后，根据自己的要求更改一些内容：

{% gallery %}
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/15.png)
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/16.png)
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/17.png)
{% endgallery %}

### 第三步——配置评论系统（giscus）

接着是修改评论方面，打开 [giscus.app/zh-CN](https://giscus.app/zh-CN)：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/18.png, alt=giscus首页, width=550px %}

完成相应要求：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/19.png, alt=giscus配置步骤1, width=550px %}

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/20.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/21.png" style="width: 48%;">
</div>

{% note warning %}
确保自己的账号是 **公开的**。
{% endnote %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/22.png, alt=检查账号公开状态, width=500px %}

打开 Settings，往下拉找到 Discussions 勾选，勾选后出现绿色的 **Set up discussions** 进行点击（账号已完成，故省略）：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/23.png, alt=Settings页面, width=500px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/24.png, alt=Discussions设置1, width=500px %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/25.png, alt=Discussions设置2, width=500px %}

{% note success %}
Discussions 打开成功！
{% endnote %}

**安装 giscus 这个 App：**

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/26.png, alt=安装giscus App, width=500px %}

**打开后进行下载和配置，完成后来到以下界面：**

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/27.png, alt=giscus配置完成, width=550px %}

**重新回到 [giscus.app/zh-CN](https://giscus.app/zh-CN) 进行仓库验证：**

{% image https://cdn.jsdelivr.net/gh/bistutzyy/tzyy/image-20241001150351940.png, alt=仓库验证, width=550px %}

**验证成功后进入 Discussion 分类：**

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/28.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/29.png" style="width: 48%;">
</div>

**启用 giscus，首先进行复制（由于不兼容可不复制后面比对修改）：**

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/30.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/31.png" style="width: 48%;">
</div>

### 第四步——谷歌数据分析

接下来是进行谷歌数据分析，打开 [analytics.google.com](https://analytics.google.com)，没有账号需自己注册 + 科学上网工具。

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/32.png, alt=Google Analytics, width=550px %}

谷歌分析展示的页面由于为手机操作效果不好，建议使用电脑，展示如下：

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/33.jpg, alt=Google Analytics面板, width=550px %}

{% span gray, 将自己的 ID 输入后完成配置操作。 %}

{% note success %}
以上便是博客的初步搭建！
{% endnote %}

---

## 附录

{% folding blue, 附1. 关于部分文件夹的讲解 %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/34.png, alt=文件夹结构讲解, width=550px %}

{% endfolding %}

{% folding green, 附2. 修改博客图标网址 %}

图标资源网站：[primer.style/foundations/icons](https://primer.style/foundations/icons/)

{% endfolding %}

{% folding purple, 附3. 关于新建文件和上传文件 %}

{% image https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/35.png, alt=新建文件入口, width=500px %}

**关于新建文件和文件夹的方法如下：**

<div style="display: flex; gap: 10px;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/36.png" style="width: 48%;">
  <img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/37.png" style="width: 48%;">
</div>

{% endfolding %}

{% folding orange, 附4. 关于谷歌账号的注册 %}

*谷歌账号注册需要科学上网工具，正常注册是需邮箱和国外手机号。*

*根据我的尝试（用的vivo手机）可以通过下载谷歌商店（Google Play）在里面注册可以跳过手机号验证。*

{% note info %}
以上只是个人尝试，不一定有用，建议上网查查解决方案。
{% endnote %}

{% endfolding %}

{% folding pink, 附5. 成果展示 %}

**下面是个人搭建博客的结果：**

{% gallery %}
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/38.png)
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/39.png)
![](https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/40.png)
{% endgallery %}

{% endfolding %}

{% folding cyan, 附6. 感悟分享 %}

{% note primary %}
学习github的搭建是对我的一个考验，当我真正想要去完成这个博客的搭建...有勇气去向王鑫学长请教问题，有毅力去完成这么一个完整的文件。在这其中自然是有方方面面的不足，但我还是很欣喜能够完成这么一个对我而言看起来高深莫测的一个方向，也让我有自信去面对学习上的挑战。感谢王鑫学长的帮助，感谢iflab学长们提供的学习机会！
{% endnote %}

{% endfolding %}

# 感谢观看！！
