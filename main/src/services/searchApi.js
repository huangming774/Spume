/** 站内搜索。GET /api/v1/search?q=&limit= → { configured, items: [{title,url,excerpt,score}] } */
export async function searchSite(query, { limit = 10, signal } = {}) {
  const params = new URLSearchParams({ q: query, limit: String(limit) });
  const res = await fetch(`/api/v1/search?${params}`, {
    headers: { Accept: "application/json" },
    signal,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || "搜索失败，请稍后再试");
  }
  return data;
}
