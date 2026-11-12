export type Lang = 'zh' | 'en';

export const STR = {
  zh: {
    blogTitle: '桃之夭夭の创作屋',
    navHome: '首页',
    navArchive: '归档',
    navArticle: '文章',
    footerLine: '桃之夭夭 © 2026',
    archiveTitle: '归档',
    archiveLead: '时光轴',
    articlesTitle: '文章',
    metaWords: '字',
    metaReads: '阅读',
    metaMinutes: '约',
    metaMinSuffix: '分钟',
    tagDone: '完成',
  },
  en: {
    blogTitle: 'Taozhiyaoyao Studio',
    navHome: 'Home',
    navArchive: 'Archives',
    navArticle: 'Posts',
    footerLine: 'Taozhiyaoyao © 2026',
    archiveTitle: 'Archives',
    archiveLead: 'Timeline',
    articlesTitle: 'Posts',
    metaWords: 'words',
    metaReads: 'reads',
    metaMinutes: '~',
    metaMinSuffix: 'min read',
    tagDone: 'Done',
  },
} as const;

export type I18nKey = keyof typeof STR.zh;
