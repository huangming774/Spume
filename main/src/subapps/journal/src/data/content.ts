import type { SiteContent } from './types';
import raw from './content.json';

export const siteContent = raw as SiteContent;

export function getArticle(slug: string) {
  return siteContent.articles.find((a) => a.slug === slug);
}
