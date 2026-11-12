export const ownerConsoleScreens = [
  {
    id: "home",
    navLabel: "总览",
    icon: "H",
    title: "站长工作台",
    subtitle: "把内容交给控制器，把构建和发布交给自动化。",
  },
  {
    id: "article",
    navLabel: "发文章",
    icon: "W",
    title: "发布文章",
    subtitle: "写博客或建站记录，提交到真实发布接口。",
  },
  {
    id: "drafts",
    navLabel: "草稿箱",
    icon: "D",
    title: "草稿箱",
    subtitle: "单独管理文章草稿和待发布内容。",
  },
  {
    id: "gallery",
    navLabel: "传相册",
    icon: "G",
    title: "相册图片",
    subtitle: "上传图片到 COS，并把图片发布到站内相册。",
  },
  {
    id: "moments",
    navLabel: "碎语",
    icon: "S",
    title: "发布碎语",
    subtitle: "填写分类和内容，直接写入首页碎语。",
  },
  {
    id: "friend",
    navLabel: "友链",
    icon: "F",
    title: "增加友链",
    subtitle: "填写信息并预览友链卡片。",
  },
  {
    id: "inbox",
    navLabel: "留言",
    icon: "M",
    title: "消息提醒",
    subtitle: "集中查看留言、朋友页评论和友链申请。",
  },
  {
    id: "emails",
    navLabel: "邮箱",
    icon: "@",
    title: "邮箱目录",
    subtitle: "只在站长后台查看注册用户邮箱和留言联系邮箱。",
  },
  {
    id: "ai",
    navLabel: "AI 状态",
    icon: "AI",
    title: "AI 调试区",
    subtitle: "把满意答案保存为用户侧固定回复。",
  },
];

export const ownerConsoleModules = [
  {
    id: "article",
    title: "发布文章",
    description: "博客文章与建站记录统一向导，写完提交真实发布接口。",
    icon: "W",
    tone: "blue",
    status: "草稿 2",
  },
  {
    id: "gallery",
    title: "相册图片",
    description: "选择相册、上传 COS，并发布到站内相册数据。",
    icon: "G",
    tone: "green",
    status: "待上传 0",
  },
  {
    id: "moments",
    title: "发布碎语",
    description: "填写分类和内容，提交到首页导航里的碎语页。",
    icon: "S",
    tone: "rose",
    status: "轻量发布",
  },
  {
    id: "friend",
    title: "增加友链",
    description: "输入站点信息，实时生成卡片预览。",
    icon: "F",
    tone: "sun",
    status: "申请 1",
  },
  {
    id: "inbox",
    title: "留言收件箱",
    description: "留言板、朋友页评论、友链申请提醒集中处理。",
    icon: "M",
    tone: "sun",
    status: "新消息 3",
  },
  {
    id: "emails",
    title: "邮箱目录",
    description: "站长后台专用，查看注册用户邮箱和留言联系邮箱。",
    icon: "@",
    tone: "blue",
    status: "后台可见",
  },
];

export const ownerConsoleNotifications = [
  {
    title: "留言板新留言",
    source: "guestbook",
    detail: "有人在留言板留了小纸条。",
    count: 1,
  },
  {
    title: "朋友页新评论",
    source: "friends",
    detail: "朋友页有新问候。",
    count: 1,
  },
  {
    title: "友链申请提醒",
    source: "申请",
    detail: "有一条待带入友链表单。",
    count: 1,
  },
];

export const ownerConsoleAvatars = [
  {
    id: "tc",
    label: "桃之夭夭控制器",
    initial: "TC",
    gradient: "linear-gradient(135deg, #7bb7ff, #83d7cf)",
  },
  {
    id: "owner",
    label: "站长",
    initial: "站",
    gradient: "linear-gradient(135deg, #ff9fbd, #ffd56f)",
  },
  {
    id: "ai",
    label: "AI",
    initial: "AI",
    gradient: "linear-gradient(135deg, #9cc9ff, #c9b8ff)",
  },
  {
    id: "gallery",
    label: "相册",
    initial: "G",
    gradient: "linear-gradient(135deg, #83d899, #83d7cf)",
  },
  {
    id: "build",
    label: "建站记录",
    initial: "B",
    gradient: "linear-gradient(135deg, #6faeff, #ffb985)",
  },
];

export const publishSteps = [
  "检查内容",
  "上传图片",
  "生成页面",
  "发布到网站",
  "检查线上结果",
];

export function getNotificationTotal(notifications = ownerConsoleNotifications) {
  return notifications.reduce((total, item) => total + (item.count || 0), 0);
}

export function buildMobileArticleDraft(inputText) {
  const material = inputText?.trim() || "今天整理了一些图片和文字素材。";
  const title = material.includes("AI")
    ? "用 AI 代理完成一次移动端发文"
    : "手机端素材整理记录";
  const body = [
    `# ${title}`,
    "",
    "这篇文章由手机端 AI 代理根据站长提供的图片和文字素材整理生成。",
    "",
    "## 素材摘要",
    material,
    "",
    "## 发布建议",
    "- 自动补全标题、摘要、标签和基础排版",
    "- 图片会在正式版上传后插入正文",
    "- 站长审核通过后再进入自动化发布流程",
  ].join("\n");

  return { title, body };
}
