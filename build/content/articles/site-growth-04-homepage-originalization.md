---
date: 2026-06-04
slug: site-growth-04-homepage-originalization
title_zh: 建站成长记录 04｜首页原创化改造：从参考学习到个人表达
title_en: Site Growth Log 04 | Homepage Originalization: From Reference Learning to Personal Expression
excerpt_zh: 这一篇记录主页原创化的一轮重要更新：从参考 GitHub 上的优秀作品出发，逐步保留自己的文案、字体、色调和叙事，再把 Hero、About、Features、Story、Footer 与移动端适配改造成更属于“桃之夭夭”的表达。
excerpt_en: This post records an important round of homepage originalization: starting from a great GitHub reference, keeping my own copywriting, typography, colors, and narrative, then reshaping Hero, About, Features, Story, Footer, and mobile behavior into something that feels more like Taozhiyy.
words: 6800
reads: 128
minutes: 26
---

# 建站成长记录 04：从“喜欢一个效果”到“长出自己的主页”

前三篇成长记录分别写了：

- 基础设施：Cloudflare、UCloud、NameSilo、GitHub Actions 以及部署链路
- 前端构成：主站、Hexo + Butterfly 博客、`build` 成长博客
- 后端能力：API、登录、留言板、AI 助手、统计与部署

这一篇想记录的是另一种成长：  
**不是网站能不能跑起来，而是这个网站能不能越来越像我自己。**

这次更新的起点其实很真实：我最开始非常喜欢 GitHub 上一个优秀前端项目的首页效果，它的动画、节奏和视觉冲击力都很强，所以我的主页最初确实参考了它的结构和表现方式。

参考项目是：

- [adrianhajdin/award-winning-website](https://github.com/adrianhajdin/award-winning-website)

但是，当网站真的部署到自己的服务器上，并且开始逐渐拥有自己的博客、成长记录、留言板、AI 助手和后台 API 之后，我也开始意识到一个问题：

> 如果这个网站以后要长期作为我的个人站点存在，它不能永远停留在“我复刻了一个我很喜欢的效果”这一层。

所以这一轮改造的目标并不是把原来的一切推倒重来，而是在最大程度保留我真正喜欢的高级动态感的基础上，逐步把主页变成更有个人表达、更有“桃之夭夭”气质的版本。

## 这次原创化不是否认参考，而是承认参考后继续往前走

我觉得这一点必须放在最前面。

这个主页一开始并不是凭空出现的。它的早期形态参考了一个非常优秀的开源学习项目，这一点我不会回避。  
相反，我认为清楚地说明参考来源，比模糊地说“都是自己做的”更加诚实，也更加尊重原作者。

所以我这次在 README 里也专门补充了中英文说明：

- 当前项目不是完全从零原创
- 首页早期阶段参考了优秀前端作品
- 我正在逐步推进原创化改造
- 本站仅用于个人学习与技术交流
- 不用于商业盈利
- 尊重原作者的劳动成果
- 涉及原项目的部分请自行查阅原作者仓库
- 我自己原创与借助 vibe coding 完成的部分，可以自由参考使用

这次主页改造的核心立场是：

> 参考可以是起点，但不能成为终点。真正的成长，是把学到的东西消化成自己的表达。

## 改造前我保留了备份

这次修改并不是盲目往前冲。  
在真正开始大改之前，我先保留了备份分支，避免改坏后无法回到原来的状态。

这一点对我来说很重要，因为主页是整个网站最显眼的位置。  
它不只是一个普通页面，而是访客打开网站时看到的第一印象。

这轮过程中保留过的备份包括：

| 备份分支 | 作用 |
| --- | --- |
| `backup/homepage-original-before-custom-redesign` | 保留开始这轮原创化前的主页状态 |
| `backup/homepage-before-originalization` | 保留更早一版主页原始状态 |
| `backup/hero-before-smooth-transition` | 保留 Hero 切换动画调整前的状态 |

这些备份让我在试错时更安心。  
因为原创化不是一次就能命中的事情，尤其是视觉交互这种东西，很多时候必须做出来、预览、觉得不对、回退、再换方向。

这也是我这次最大的感受之一：

> 有备份，才敢认真试错；敢试错，才有可能真正做出自己的东西。

## 哪些内容我选择保留

原创化并不等于把所有东西都删掉。  
这次改造里，我特别强调了一点：**我自己原来写好的文案、字体和站点气质要保留。**

比如首页左上角的：

- `WELCOME`

以及右下角的：

- `桃之夭夭`

这些文字不是参考项目里的内容，而是我已经改过、写过、调过的个人表达。  
它们和我的站点名字、头像、博客主题、整体风格是一体的。

所以这一轮并不是为了“改而改”，而是分清楚：

- 哪些是我自己的，要保留
- 哪些是参考项目痕迹太重的，要重做
- 哪些效果我仍然喜欢，但需要换成自己的实现方式
- 哪些页面节奏已经适合我，只需要增加新的原创交互

这比简单重写更难一点。  
因为完全推倒重来很容易变成另一个模板，而保留自己喜欢的部分再一点点脱离参考痕迹，反而更考验判断。

## 第一部分：Hero 从图片切换变成“头像指南针”

这次最重要、也最明显的变化，就是首页 Hero 区域。

原来的 Hero 图片切换效果虽然很顺滑，也很有高级感，但它和参考项目的视觉结构仍然比较接近。  
如果只是换图片、换文字，其实还不够。因为访客感受到的交互节奏和视觉语言仍然像原项目。

所以我一开始尝试过几种方向：

- 更顺滑的图片过渡
- 花瓣切换
- 立体卡牌选择
- 壁纸卡片选择
- 指南针式选择轮盘

这些方向并不是每个都成功。  
比如花瓣切换虽然听起来有“桃之夭夭”的感觉，但做出来后效果不够自然，反而有点像为了原创而硬加装饰。  
立体卡牌也遇到了横屏壁纸放进竖屏卡片后主体显示不完整的问题。

最后真正稳定下来的，是“头像指南针”这一版。

### 现在的 Hero 逻辑

现在访客点击 `CHANGE` 后，不再只是进入普通图片切换，而是打开一个带有方位感的选择器：

- 中心是我的头像圆形徽章
- 外层有透明罗盘结构
- 方位标识围绕头像分布
- 四张横屏壁纸对应不同方向
- 访客可以操控罗盘
- 图片位置和方位会跟随移动变化
- 退出后再次打开会复位
- 页面提示访客“转动指南针有惊喜”

这个变化让 Hero 不再只是“几张图片轮播”，而变成一个更有个人站点气质的小互动。

它同时结合了几个属于我自己的元素：

- 头像
- 桃之夭夭的轻盈感
- 方位与探索感
- 访客主动操控
- 横屏壁纸主体展示

这比单纯修改动画曲线更有原创性。  
因为它改变的不只是动画外观，而是改变了交互结构本身。

### 为什么不用自动旋转

中间也尝试过让图片或罗盘自动缓慢移动。  
但后来我觉得更适合的方向是：**由访客控制，而不是页面自己表演。**

原因有两个。

第一，自动旋转容易让页面看起来很热闹，但访客其实只是被动观看。  
第二，指南针这个隐喻本身就很适合“由人来转动”。如果它自己一直转，就少了探索的手感。

所以最终版本更偏向：

- 页面给出提示
- 访客主动进入
- 访客转动罗盘
- 页面跟随反馈
- 选择某个方向后切换壁纸

这种方式更像一个小小的“站点入口仪式”。

### 罗盘头像资源为什么改成腾讯云 COS

上线后我发现一个问题：新部署的罗盘中心图片没有加载出来。  
这说明本地图片路径在部署环境里并不稳定，或者没有被正确打包和引用。

所以最后把罗盘头像资源上传到了腾讯云 COS，并改为使用线上链接：

| 资源 | 处理方式 |
| --- | --- |
| 罗盘中心头像 | 上传到腾讯云 COS |
| 前端引用 | 使用 COS 静态链接 |
| 目的 | 确保部署后也能稳定加载 |

这个问题也提醒我：  
前端视觉资产只在本地能看到不算完成，真正部署到服务器后仍然能稳定加载，才算完成。

## 第二部分：About 从展示图变成更有叙事感的视觉区域

About 这一部分原来有一个“小屏展现大屏”的视觉结构，我其实很喜欢它。  
它有一种从局部走向完整画面的感觉，很适合做个人站点里的“介绍区”。

但如果完全保留原来的视觉逻辑，它仍然会有参考痕迹。  
所以这次思路不是删掉，而是把它改成更偏个人表达的展示。

这一块调整的重点包括：

- 保留图片从局部到整体的视觉张力
- 减少过于像参考项目的结构
- 改善手机端图片显示不完整的问题
- 调整滑动缩放时的节奏
- 让移动端也保留动态效果
- 减少移动端不必要的大空白

最开始移动端适配并不好。  
图片显示不完整，滑动扩展也有问题，甚至一度出现了空白过多、粉色装饰残留等情况。

后面重新处理后，这一部分的目标变成：

> 桌面端保留视觉推进感，手机端不要硬压缩桌面结构，而是用更适合小屏的节奏重新呈现。

这也是前端适配里很重要的一点。  
手机端不是把桌面端缩小，而是重新组织注意力。

## 第三部分：Features 保留主排版，新增“档案书”交互

Features 这一部分我原本就很满意。  
它的排版、层级、节奏都比较符合我对主页展示区域的预期，所以我并不想把它完全推翻。

但问题也在这里：  
如果完全不改，它的结构仍然会显得和参考项目太接近。

所以最后采用的方案是：

> 主展示保持为主，新增一个访客点击后才出现的档案书交互。

也就是说，默认情况下，访客看到的仍然是我喜欢的 Features 排版。  
但页面额外提供一个“收录 / 档案书”式入口，让这些内容可以用另一种方式被查看。

### 为什么是档案书

档案书这个概念很适合个人网站。  
因为 Features 本质上是在展示这个站点里已经长出来的东西：

- 博客
- 成长记录
- 留言板
- AI 助手
- 后端能力
- ACG 内容
- 子页面与站点工具

这些东西不像商业产品的功能清单，更像是一个个人创作项目不断积累的“档案”。  
所以用“档案书”来呈现，比简单做一个按钮弹窗更贴合站点气质。

### 这部分经历了几次调整

一开始我想做五个立体卡片翻页式的效果，但很快遇到问题：

- 有几张图是横屏素材
- 放进卡片后裁剪不自然
- 空白区域偏多
- 整体显得有点奇怪
- 装饰元素一度太像普通贴纸

后来继续调整思路，把它从“强行做一个很复杂的书”改成更轻量的“档案式入口”。  
这样主排版仍然是核心，档案书是增强体验，而不是抢走整个区域的主角位置。

这也让我意识到：

> 原创不是元素越多越原创，而是每个新增元素都要服务这个网站本身。

## 第四部分：Feature 与 Story 的背景从纯黑走向浅色调

之前主页有比较强烈的黑色背景和高对比视觉。  
这种风格确实很酷，也很适合做冲击力强的首页。

但随着 Hero 改成头像罗盘、Features 增加档案书、Story 改成纸笺感之后，纯黑色背景就开始显得没有那么合适。

所以这次把 Features 以及下面 Story 区域的背景色调调整成更浅、更柔和的方向。

这个变化的意义不只是“换颜色”，而是统一新的视觉气质：

- 更轻
- 更温柔
- 更适合桃之夭夭
- 更像个人创作站
- 更少游戏宣传片式的压迫感

对于我的网站来说，浅色并不代表没有高级感。  
关键是要通过层次、留白、动画和材质感来撑住，而不是只靠黑底大图制造冲击。

## 第五部分：Story 从大图展示变成纸笺与信件感

Story 这一部分我也很喜欢原来的不规则裁剪。  
它和我的图片很搭，有一种画面从边缘露出来的感觉。

所以这里的目标不是把不规则设计删掉，而是把它重新解释成更属于我的语义：

> 不只是视觉大图，而像一封写给访客的信，一张留在页面里的纸笺。

这次调整里，Story 逐步向“信件式 / 纸笺式”靠近：

- 保留不规则裁剪带来的灵动感
- 加入更像纸张和信件的视觉层次
- 让文字区域更有阅读感
- 调整纸笺文字大小
- 避免文字刚好卡在两个色调交界处
- 移动端恢复更适合小屏的图片展示

这部分也经历过回退和重改。  
第一次改出来后效果并不对，纸笺感和原来的图片结构没有融合好，后来才重新调整。

最后保留下来的思路是：  
**不规则裁剪是视觉记忆点，纸笺语义是原创方向。**

这样既没有完全丢掉我喜欢的画面感，也不再只是沿用原来的大图展示逻辑。

## 第六部分：页脚改成浅色，更贴合整体氛围

页脚内容包括：

- `@bistutzyy`
- GitHub
- Bilibili
- Vercel
- 邮箱
- “本站仅作学习使用，感谢开源”

原来页脚在深色背景下比较自然，但当 Features 和 Story 都转向浅色调后，页脚如果仍然保持深色或对比过强，就会显得断层。

所以这次也把页脚颜色改成浅色方向。  
这个调整虽然小，但它对整体完成度很重要。

一个页面是否“像一个整体”，很多时候不是看最大的 Hero，而是看这些边角区域有没有跟上主视觉的变化。

## 第七部分：移动端不是附带任务，而是必须重做节奏

这轮改造里，手机端花了不少时间。  
因为很多桌面端看起来不错的效果，到了手机端会立刻暴露问题。

这次主要处理了：

| 区域 | 移动端问题 | 调整方向 |
| --- | --- | --- |
| About | 图片显示不完整，滑动扩展不自然 | 重新处理小屏展示与动态效果 |
| Features | 档案书和卡片空间不协调 | 降低装饰干扰，避免空白过多 |
| Story | 图片展示不符合预期 | 按更接近原本手机端节奏重做 |
| Feature 到 Story 过渡 | 空白太多，出现多余粉色元素 | 收紧间距，隐藏不必要装饰 |
| Hero 罗盘 | 需要复位和同步方位 | 退出后重置，移动过程同步方向 |

这让我更明确地感觉到：  
如果一个首页只在电脑端漂亮，它还不能算真正完成。

尤其个人站点的访客很可能来自手机。  
手机端打开时，页面如果空白太多、图片裁剪奇怪、动画缺失，整体感觉会立刻下降。

所以这次移动端调整不只是“修 bug”，也是原创化的一部分。  
因为响应式体验本身，也会决定这个网站是不是一个真正被认真打磨过的作品。

## 第八部分：这次改造具体改变了什么

为了更直观地记录，这里放一张改造对照表。

| 模块 | 原来的问题 | 这次原创化方向 | 当前结果 |
| --- | --- | --- | --- |
| Hero | 图片切换结构仍接近参考项目 | 改成头像指南针选择器 | 访客可操控罗盘选择壁纸 |
| Hero 资源 | 本地图片部署后可能丢失 | 上传腾讯云 COS | 罗盘头像线上稳定加载 |
| 首页文案 | 有些文案是我自己的 | 保留原创文字与字体 | `WELCOME`、`桃之夭夭` 等继续保留 |
| About | 视觉效果喜欢但参考感仍在 | 调整展示节奏与移动端表现 | 更像个人站展示区域 |
| Features | 排版满意但需要增强原创性 | 默认主排版 + 档案书入口 | 主次关系更清楚 |
| Story | 喜欢不规则裁剪但要换语义 | 纸笺 / 信件式重构 | 保留灵动感，增加个人叙事 |
| Footer | 深色与新背景不协调 | 改成浅色 | 更统一 |
| Mobile | 多处新效果未完全适配 | 重做小屏节奏 | About、Features、Story 更自然 |
| README | 需要说明参考与原创边界 | 中英双语说明 | 更清楚表达立场 |

这张表其实也说明了我对“原创化”的理解：

> 原创化不是只看某一个按钮、某一个动画，而是看整体体验、交互结构、文案气质、资源管理和维护方式有没有逐渐变成自己的。

## 这次能不能算原创？

这个问题我自己也反复问过。

如果说“百分百从零原创”，那当前项目还不能这样说。  
因为早期首页确实参考了一个优秀项目，很多学习过程也建立在那个项目给我的启发之上。

但如果说“这次改造是否已经具备明显原创表达”，我认为答案是肯定的。

原因是这轮改造已经不只是：

- 换图片
- 换文字
- 换颜色
- 改一点动画参数

而是改了很多更深层的东西：

- Hero 的交互结构变了
- 访客与页面的关系变了
- 头像成为核心视觉符号
- Features 增加了档案书式收录逻辑
- Story 从视觉展示转向纸笺叙事
- 浅色调统一了新的主页气质
- 移动端体验重新调整
- README 明确了参考、原创、非商业和尊重来源

所以我更愿意把现在的状态定义为：

> 这是一个从参考学习阶段进入原创化改造阶段的个人主页。

它还不是完全脱离所有参考痕迹的终点，但已经不再只是简单复刻。  
它正在长出自己的视觉系统、交互记忆点和叙事方式。

## 非商业使用并不等于可以忽略尊重

这个网站目前部署在我自己的服务器上，用于个人学习和技术交流，不用于商业盈利。

但我也知道：  
非商业并不代表可以完全不在意原作者的权益。

所以我现在采取的方式是：

- README 明确说明参考项目
- 保持对原作者的感谢
- 不把参考项目包装成完全原创
- 持续推进原创化
- 涉及原项目的部分提醒读者去原作者仓库查阅
- 自己原创和 vibe coding 完成的部分单独说明可参考使用

这也是我希望以后长期保持的态度。

我不想用“学习项目”当作模糊边界的借口。  
更好的做法是：承认来源，感谢来源，然后继续把它做成自己的东西。

## 这次我学到的东西

这轮改造其实不只是做页面，也让我学到了很多判断。

### 1. 喜欢一个效果，不代表必须照着它走到底

最开始我选择那个参考项目，就是因为它的动效真的很合眼。  
但后来我意识到，喜欢它的高级感，不等于必须保留它的每一个结构。

我真正想保留的是：

- 顺滑
- 高级
- 有沉浸感
- 有探索感
- 有视觉记忆点

而不是某一个固定动画。

### 2. 原创化要改“结构”，不只是改“皮肤”

如果只改颜色和图片，访客仍然能感觉到原项目的影子。  
真正让页面变成自己的，是交互逻辑和叙事方式发生变化。

这就是为什么 Hero 最后要改成指南针，而不是继续做普通切换。

### 3. 回退不是失败

这次中间回退过几次：

- 花瓣切换不合适，回退
- 纸笺 Story 第一次不对，重改
- 档案书空白太多，继续调整
- 手机端效果不对，重新适配

这些都不是浪费。  
它们让我更清楚什么不适合这个网站。

### 4. 个人站点最重要的是气质一致

功能可以很多，页面也可以复杂，但最后必须回到一个问题：

> 这是不是像我的网站？

这次改造之后，Hero 的头像罗盘、浅色背景、纸笺 Story、档案书 Features、页脚学习声明，都更接近我想要的个人站气质。

## 下一步还可以继续怎么做

这次更新不是原创化的终点。  
后面我还可以继续往几个方向推进。

| 方向 | 目标 |
| --- | --- |
| 继续统一视觉系统 | 让主站、build、blog 的气质更协调 |
| 优化 Features 档案书 | 让档案书更像真正的收录系统 |
| 增强移动端细节 | 继续减少小屏不必要空白 |
| 整理素材来源 | 让图片、头像、图标资源更清晰 |
| 继续减少参考痕迹 | 逐步把剩余结构改成自己的表达 |
| 写更多成长记录 | 把每一次大更新都留下过程 |

如果说前三篇记录的是“网站怎么搭起来”，那这一篇记录的就是：

> 网站开始从工程项目，变成一个更像我自己的创作空间。

这件事对我来说很重要。  
因为一个个人网站真正有生命力的地方，不只是它部署在哪里、用了什么技术栈，而是它有没有慢慢长出自己的语言。

系列上一篇：

- [建站成长记录 03｜后端实现：接口、登录、留言板与 AI 助手](https://taozhiyy.top/build/post/site-growth-03-backend-services)

---en---

# Site Growth Log 04: From Liking an Effect to Growing My Own Homepage

The first three growth logs covered:

- infrastructure: Cloudflare, UCloud, NameSilo, GitHub Actions, and deployment
- frontend structure: the main site, the Hexo + Butterfly blog, and the `build` growth blog
- backend capabilities: APIs, login, guestbook, AI assistant, statistics, and deployment

This fourth post records a different kind of growth:  
**not whether the site can run, but whether it can increasingly feel like mine.**

The starting point of this update was honest and simple. I really liked the homepage effects of an excellent GitHub frontend project. Its animation, rhythm, and visual impact were strong, so the early version of my homepage did reference its structure and presentation.

The reference project is:

- [adrianhajdin/award-winning-website](https://github.com/adrianhajdin/award-winning-website)

But once the site was actually deployed on my own server and started growing its own blog, growth log, guestbook, AI assistant, and backend API, I began to face a more serious question:

> If this website is going to exist as my long-term personal site, it cannot stay forever at the level of “I recreated an effect I liked.”

So the goal of this round was not to throw everything away.  
The goal was to keep the advanced motion and atmosphere I truly liked, while gradually turning the homepage into something with stronger personal expression and a clearer Taozhiyy identity.

## Originalization is not denying the reference

I want to put this first.

The homepage did not appear from nowhere. Its early form was inspired by a very strong learning project. I do not want to hide that.  
In fact, clearly stating the source is more honest and more respectful than vaguely claiming that everything was entirely original.

That is why I also updated the README with bilingual notes:

- the current project is not completely original from scratch
- the homepage was inspired by an excellent frontend project in its early stage
- I am gradually pushing the project toward originalization
- the site is for personal learning and technical exchange
- it is not used for commercial profit
- I respect the original author's work
- parts related to the original project should be checked in the original repository
- parts created by me or with vibe coding can be freely referenced and used

The core position of this redesign is:

> A reference can be the starting point, but it should not become the destination. Real growth means digesting what I learned and turning it into my own expression.

## I kept backups before changing the homepage

This redesign was not done blindly.  
Before making major changes, I kept backup branches so I could safely return to previous states.

That matters because the homepage is the most visible part of the whole site.  
It is not just another page. It is the first impression visitors get.

The backup branches included:

| Backup branch | Purpose |
| --- | --- |
| `backup/homepage-original-before-custom-redesign` | preserves the homepage before this round of originalization |
| `backup/homepage-before-originalization` | preserves an earlier homepage state |
| `backup/hero-before-smooth-transition` | preserves the Hero state before transition experiments |

These backups made the design process safer.  
Originalization is rarely correct on the first try, especially for visual interaction. Many ideas need to be built, previewed, judged, reverted, and tried again.

One of my biggest lessons from this round is:

> Backups make serious experimentation possible. Serious experimentation is what gives a project a chance to become truly personal.

## What I deliberately kept

Originalization does not mean deleting everything.

In this redesign, I emphasized one thing very clearly:  
**my own copywriting, typography, and site atmosphere should remain.**

For example, the top-left:

- `WELCOME`

and the bottom-right:

- `桃之夭夭`

These are not from the reference project. They are words I had already written, adjusted, and made part of the site's identity.  
They belong together with the site name, avatar, blog theme, and overall tone.

So this redesign was not about changing things just for the sake of change.  
It was about separating:

- what is mine and should be kept
- what still feels too close to the reference and should be rebuilt
- what effect I still like but need to implement differently
- what rhythm already works and only needs new original interaction

This is harder than a full rewrite.  
Starting over can easily become just another template. Keeping what matters while removing the reference traces requires more careful judgment.

## Part 1: Hero became an avatar compass

The most important and visible change is the Hero section.

The old Hero image switching felt smooth and premium, but its visual structure was still close to the reference project.  
Changing only the images or text was not enough, because the visitor would still feel a similar interaction rhythm and visual language.

During this round, I tried several directions:

- a smoother image transition
- petal-style switching
- 3D card selection
- wallpaper card selection
- a compass-style selection wheel

Not all of them worked.  
The petal idea sounded fitting for “Taozhiyy,” but the actual result felt forced rather than natural.  
The 3D card idea also ran into problems because horizontal wallpapers did not fit well into vertical card frames.

The version that finally stayed was the avatar compass.

### How the Hero works now

When visitors click `CHANGE`, they no longer just enter a normal image switch.  
Instead, they open a directional selector:

- the center is my circular avatar badge
- a transparent compass layer surrounds it
- direction labels are placed around the avatar
- four horizontal wallpapers correspond to different directions
- visitors can control the compass
- image position and direction labels move together
- reopening the selector resets it
- the page hints that turning the compass may bring a surprise

This turns the Hero from “a few rotating images” into a small interaction that belongs more clearly to this personal site.

It combines several elements that are mine:

- my avatar
- the light Taozhiyy atmosphere
- direction and exploration
- active visitor control
- horizontal wallpaper presentation

That is more original than simply adjusting an animation curve.  
It changes not just the look of the animation, but the interaction structure itself.

### Why it is not automatic rotation

At one point, I considered making the images or compass move slowly by themselves.  
But the better direction was: **let visitors control it instead of making the page perform by itself.**

There are two reasons.

First, automatic motion can make a page look busy while visitors remain passive.  
Second, the compass metaphor naturally asks to be turned by a person. If it spins on its own, the sense of exploration becomes weaker.

So the final version is closer to:

- the page gives a hint
- the visitor opens the selector
- the visitor turns the compass
- the page responds
- a direction is chosen and the wallpaper changes

It feels more like a small entrance ritual for the site.

### Why the compass avatar uses Tencent COS

After deployment, I noticed that the new compass center image did not load.  
That meant the local image path was not stable in the deployed environment or was not being packaged correctly.

So I uploaded the compass avatar image to Tencent Cloud COS and changed the frontend to use the online URL.

| Resource | Handling |
| --- | --- |
| Compass center avatar | uploaded to Tencent COS |
| Frontend reference | uses a COS static link |
| Goal | stable loading after deployment |

This also reminded me of an important rule:  
a visual asset is not finished just because it works locally. It is finished only when it still loads reliably after deployment.

## Part 2: About became more narrative

The About section originally had a “small screen revealing a large screen” structure. I actually liked that effect.  
It had a feeling of moving from a partial view toward a larger picture, which fits a personal site introduction well.

But if I kept the exact visual logic, the reference trace would still be too visible.  
So instead of deleting it, I reshaped it into a more personal display.

The key adjustments were:

- keeping the visual tension between partial and full image
- reducing the similarity to the reference structure
- improving incomplete image display on mobile
- tuning the scroll and scale rhythm
- preserving motion on mobile
- reducing unnecessary empty space on small screens

The first mobile version was not good enough.  
The image was incomplete, the scroll expansion felt wrong, and there were moments with too much blank space or leftover pink decoration.

After reworking it, the goal became:

> Keep the visual progression on desktop, but do not simply shrink the desktop structure on mobile. Rebuild the rhythm for small screens.

That is an important lesson in frontend adaptation.  
Mobile is not just a smaller desktop. It is a different organization of attention.

## Part 3: Features kept the main layout and gained an archive book

I was already very satisfied with the Features section.  
Its layout, hierarchy, and rhythm fit what I wanted from a homepage showcase area, so I did not want to completely replace it.

But that also created a problem:  
if I did not change it at all, its structure would still feel too close to the reference.

So the final solution was:

> Keep the main display as the primary experience, and add an archive-book interaction that appears when visitors choose to open it.

In other words, the default view still keeps the Features layout I like.  
But the page now provides an additional “collection / archive book” entry so the same content can be viewed in a different way.

### Why an archive book

The archive-book idea fits a personal site well.  
Features are not just a commercial product checklist. They are things this project has gradually grown:

- blog
- growth log
- guestbook
- AI assistant
- backend services
- ACG content
- subpages and site tools

These are closer to accumulated records than product features.  
So presenting them as an archive feels more natural than using a simple modal button.

### How this section evolved

At first, I wanted five 3D flipping cards, but problems appeared quickly:

- some source images were horizontal
- card cropping looked unnatural
- empty areas became too large
- the overall composition felt strange
- decorative elements once looked too much like ordinary stickers

Later, the direction changed from “forcing a complex book effect” to a lighter archive-style entrance.  
That way, the main layout remains the core, and the archive book becomes an enhancement instead of stealing the entire section.

This taught me another thing:

> Originality does not mean adding more elements. Each new element must serve the site itself.

## Part 4: Feature and Story backgrounds moved from black to light tones

The previous homepage used a strong black background and high contrast.  
That style is cool and works well for a very cinematic landing page.

But after the Hero became an avatar compass, Features gained an archive layer, and Story moved toward a paper-note feeling, the pure black background started to feel less suitable.

So I changed the Features and Story areas toward a lighter and softer palette.

This was not just a color change.  
It helped unify the new visual direction:

- lighter
- softer
- more suitable for Taozhiyy
- more like a personal creative site
- less like a game trailer landing page

For this website, light colors do not mean losing quality.  
The key is to support the design with layers, spacing, motion, and material feeling, rather than relying only on dark backgrounds and large images.

## Part 5: Story became a letter-like paper note

I also liked the irregular crop in the original Story section.  
It worked well with my image and created a feeling of an image peeking out from the page edge.

So the goal here was not to delete the irregular design.  
The goal was to reinterpret it with a more personal meaning:

> not just a big visual image, but like a letter or paper note left for the visitor.

The Story section gradually moved toward a letter / paper-note style:

- keeping the liveliness of irregular cropping
- adding visual layers that feel more like paper
- making the text area more readable
- increasing the note text size
- avoiding awkward color-boundary placement
- restoring a better mobile image presentation

This part also went through a revert and rework.  
The first attempt did not combine the paper feeling and image structure well enough, so it had to be redesigned.

The final idea is:

**irregular cropping is the visual memory point, while the paper-note metaphor is the original direction.**

That way, I did not lose the image effect I liked, but I also moved away from the original big-image presentation logic.

## Part 6: Footer moved to a light tone

The footer includes:

- `@bistutzyy`
- GitHub
- Bilibili
- Vercel
- email
- “This site is for learning only. Thanks to open source.”

The old footer worked naturally on a dark background.  
But once Features and Story became lighter, a dark or overly high-contrast footer began to feel disconnected.

So I also changed the footer colors toward a lighter direction.

This may seem small, but it matters for overall completeness.  
Whether a page feels like one complete design is often decided not only by the Hero, but also by these edge areas.

## Part 7: Mobile was not an afterthought

Mobile adaptation took a lot of time in this round.  
Many effects that looked good on desktop immediately exposed problems on small screens.

The main mobile issues included:

| Area | Mobile issue | Adjustment |
| --- | --- | --- |
| About | incomplete image display and awkward scroll expansion | rebuilt small-screen display and motion |
| Features | archive book and cards did not fit well | reduced decorative interference and empty space |
| Story | image display did not match expectation | restored a more suitable mobile rhythm |
| Feature-to-Story transition | too much blank space and extra pink elements | tightened spacing and hid unnecessary decoration |
| Hero compass | needed reset and direction synchronization | resets after exit and keeps direction synced |

This made one thing clearer:

If a homepage only looks good on desktop, it is not really finished.

Many visitors will open a personal site on mobile.  
If the mobile version has excessive blank space, strange cropping, or missing motion, the whole experience drops immediately.

So mobile work was not just bug fixing. It was part of the originalization process.  
Responsive behavior also determines whether the site feels genuinely crafted.

## Part 8: What actually changed

Here is a clearer comparison table.

| Module | Previous issue | Originalization direction | Current result |
| --- | --- | --- | --- |
| Hero | image switch still felt close to the reference | avatar compass selector | visitors control a compass to choose wallpapers |
| Hero asset | local image could fail after deployment | upload to Tencent COS | compass avatar loads from an online asset |
| Homepage copy | some text was already mine | preserve original text and typography | `WELCOME`, `桃之夭夭`, and related tone remain |
| About | liked the effect but reference trace remained | adjust rhythm and mobile display | feels more like a personal introduction area |
| Features | layout worked but needed more originality | default layout plus archive-book entry | clearer primary-secondary relationship |
| Story | liked irregular crop but needed new meaning | paper-note / letter-like redesign | keeps liveliness and adds narrative |
| Footer | dark tone no longer matched | switch to light tone | more unified |
| Mobile | several new effects were not fully adapted | rebuild small-screen rhythm | About, Features, and Story are more natural |
| README | reference and originality boundary needed clarity | bilingual explanation | clearer position and respect for sources |

This table shows how I understand originalization:

> Originalization is not just one button or one animation. It is whether the overall experience, interaction structure, copywriting, resource handling, and maintenance thinking gradually become my own.

## Can this be considered original now?

I asked myself this question many times.

If “original” means 100% built from nothing, then the current project should not be described that way.  
The early homepage did reference an excellent project, and my learning process was helped by that project.

But if the question is whether this round now has clear original expression, I think the answer is yes.

Because this update is not only:

- changing images
- changing words
- changing colors
- adjusting a few animation parameters

It changed deeper things:

- the Hero interaction structure changed
- the relationship between visitors and the page changed
- my avatar became a core visual symbol
- Features gained an archive-book collection logic
- Story moved from visual display to paper-note narrative
- the light palette unified the new homepage atmosphere
- mobile experience was reworked
- README now clearly states reference, original work, non-commercial use, and source respect

So I would describe the current state as:

> a personal homepage that has moved from reference learning into active originalization.

It is not the final point where every reference trace has disappeared, but it is no longer a simple recreation.  
It is growing its own visual system, interaction memory points, and narrative language.

## Non-commercial use does not remove the need for respect

This site is deployed on my own server for personal learning and technical exchange. It is not used for commercial profit.

But I also understand that non-commercial use does not mean source respect can be ignored.

So my current approach is:

- clearly state the reference project in README
- keep gratitude toward the original author
- do not present the reference-based parts as completely original
- continue pushing originalization forward
- guide readers to the original repository for original-project-related parts
- separately state that my own and vibe-coding-assisted parts can be referenced freely

This is the attitude I want to keep long term.

I do not want to use “learning project” as a vague excuse.  
A better way is to acknowledge the source, thank it, and then keep building the site into something of my own.

## What I learned from this round

This redesign was not only about the page. It also taught me how to judge visual work better.

### 1. Liking an effect does not mean following it forever

I chose the reference project at first because its motion really matched my taste.  
But later I realized that liking its premium feeling does not mean keeping every structure.

What I truly wanted to keep was:

- smoothness
- quality
- immersion
- exploration
- visual memory

not one fixed animation.

### 2. Originalization should change structure, not only skin

If I only changed colors and images, visitors would still feel the original project's shadow.  
What makes the page more mine is a change in interaction logic and narrative.

That is why the Hero finally became a compass instead of remaining a normal transition.

### 3. Reverting is not failure

This round included several reversions:

- petal switching did not fit, so it was reverted
- the first Story paper-note version was wrong, so it was reworked
- the archive book had too much blank space, so it was adjusted
- the mobile version was not good enough, so it was adapted again

None of that was wasted.  
Each failed direction made it clearer what did not fit this site.

### 4. A personal site needs a consistent atmosphere

There can be many features and many pages, but the final question is:

> Does this feel like my website?

After this round, the avatar compass, light background, paper-note Story, archive-style Features, and footer learning statement all feel closer to the personal-site atmosphere I want.

## What comes next

This update is not the endpoint of originalization.  
There are still several directions I can continue:

| Direction | Goal |
| --- | --- |
| unify the visual system | make the main site, build site, and blog feel more coherent |
| refine the archive book | make it feel more like a real collection system |
| improve mobile details | reduce unnecessary empty space on small screens |
| organize visual assets | make images, avatars, and icons easier to trace |
| reduce remaining reference traces | gradually replace remaining structures with my own expression |
| write more growth logs | record every major update as part of the site's history |

If the first three posts recorded how the site was built, this fourth post records something more personal:

> the website is starting to move from an engineering project into a creative space that feels more like me.

That matters to me.  
Because the real life of a personal website is not only where it is deployed or what tech stack it uses. It is whether it slowly grows its own language.

Previous in the series:

- [Site Growth Log 03 | Backend Implementation for APIs, Login, Guestbook, and the AI Assistant](https://taozhiyy.top/build/post/site-growth-03-backend-services)
