import { mockBangumiList } from "../data/acgMock.js";
import { apiUrl } from "../lib/apiBase.js";
import { asList } from "../lib/asList.js";

async function fetchJson(path, { signal } = {}) {
  const res = await fetch(apiUrl(path), {
    headers: { Accept: "application/json" },
    signal,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const ct = res.headers.get("content-type") || "";
  if (!ct.includes("application/json")) {
    throw new Error("not json");
  }
  return res.json();
}

export async function getBangumiList() {
  try {
    const data = await fetchJson("/api/v1/bangumi/list");
    const items = asList(data);
    return items.length > 0 ? items : mockBangumiList;
  } catch {
    return mockBangumiList;
  }
}

export async function getRadarFeed() {
  try {
    const data = await fetchJson("/api/v1/radar/feed");
    const items = asList(data);
    return items;
  } catch {
    return [];
  }
}

export async function getWallpaperGift({ apiOnly = false, signal } = {}) {
  const data = await fetchJson(
    apiOnly ? "/api/v1/wallpapers/draw?source=api" : "/api/v1/wallpapers/draw",
    { signal }
  );
  return data?.item || null;
}
