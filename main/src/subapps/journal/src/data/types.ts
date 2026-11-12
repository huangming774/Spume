export type Article = {
  slug: string;
  date: string;
  modified?: string;
  title_zh: string;
  title_en: string;
  excerpt_zh: string;
  excerpt_en: string;
  words: string;
  reads: string;
  minutes: string;
  bodyHtmlZh: string;
  bodyHtmlEn: string;
};

export type SiteContent = {
  articles: Article[];
};
