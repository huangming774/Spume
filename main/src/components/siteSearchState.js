/** 搜索面板的纯逻辑部分，方便单测。 */

/** 查询词去空白后非空才值得发请求。 */
export function isSearchable(query) {
  return typeof query === "string" && query.trim().length > 0;
}

/** 后端命中转成面板行：没有标题或地址的丢掉，摘要截到 80 字。 */
export function normalizeHits(items) {
  if (!Array.isArray(items)) return [];
  return items
    .filter(
      (item) =>
        item &&
        typeof item.title === "string" &&
        typeof item.url === "string" &&
        item.title.trim() &&
        item.url.trim()
    )
    .map((item) => ({
      title: item.title.trim(),
      url: item.url.trim(),
      excerpt:
        typeof item.excerpt === "string" ? item.excerpt.trim().slice(0, 80) : "",
    }));
}
