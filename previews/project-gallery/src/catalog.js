export const projects = [];

export function filterProjects(items, query = '') {
  const term = query.trim().toLocaleLowerCase();
  return items.filter(item => !term || [item.title, item.summary, item.author, item.category, ...item.tags].join(' ').toLocaleLowerCase().includes(term));
}
export function nextProject(items, id, direction) {
  if (!items.length) return null;
  const index = items.findIndex(item => item.id === id);
  return items[index < 0 ? 0 : (index + direction + items.length) % items.length];
}
export function projectPage(items, page = 0, size = 12) {
  const limit = Math.max(1, Math.floor(size) || 12);
  const pages = Math.ceil(items.length / limit);
  const current = Math.max(0, Math.min(Math.floor(page) || 0, Math.max(0, pages - 1)));
  return {items:items.slice(current * limit, (current + 1) * limit), page:current, pages, total:items.length};
}
