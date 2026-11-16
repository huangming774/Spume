import { useEffect, useState } from "react";

export async function loadEntries(kind) {
  try {
    const response = await fetch(`/api/content/${encodeURIComponent(kind)}`);
    if (!response.ok) return [];
    const data = await response.json();
    return Array.isArray(data.items) ? data.items : [];
  } catch {
    return [];
  }
}

export function useEntries(kind) {
  const [state, setState] = useState({ items: [], loaded: false });
  useEffect(() => {
    let active = true;
    const load = () => {
      loadEntries(kind).then((next) => {
        if (active) setState({ items: next, loaded: true });
      });
    };
    load();
    window.addEventListener("site-entries-changed", load);
    return () => {
      active = false;
      window.removeEventListener("site-entries-changed", load);
    };
  }, [kind]);
  return state;
}

export async function loadRecentEntries() {
  try {
    const response = await fetch("/api/content");
    if (!response.ok) return [];
    const data = await response.json();
    return Array.isArray(data.items) ? data.items : [];
  } catch {
    return [];
  }
}

const UPDATE_LINKS = {
  article: { tag: "文章", href: (item) => `/blog/${item.id}`, action: "阅读" },
  journal: { tag: "建站日志", href: (item) => `/build/post/${item.id}`, action: "查看记录" },
  about: { tag: "关于我", href: () => "/about", action: "去看看" },
  gallery: { tag: "浮光集", href: () => "/glimmer", action: "打开相册" },
  moment: { tag: "碎语", href: () => "/glimmer", action: "去看看" },
  bili: { tag: "哔哩", href: () => "/bili", action: "去看看" },
  project: { tag: "星空馆", href: () => "/projects", action: "走进星空" },
};

export function asConsoleUpdates(entries) {
  return entries.map((entry) => {
    const link = UPDATE_LINKS[entry.kind] || {
      tag: "更新",
      href: () => "/",
      action: "去看看",
    };
    const text = String(entry.body || "")
      .replace(/\s+/g, " ")
      .trim();
    return {
      id: entry.id,
      date: String(entry.createdAt || "").slice(0, 10),
      tag: link.tag,
      title: entry.title || link.tag,
      text: text ? text.slice(0, 42) : "已发布",
      href: link.href(entry),
      action: link.action,
    };
  });
}

export function notifyEntriesChanged() {
  window.dispatchEvent(new Event("site-entries-changed"));
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function bodyHtml(body, imageUrl) {
  const parts = [];
  if (imageUrl) parts.push(`<p><img src="${escapeHtml(imageUrl)}" alt=""></p>`);
  String(body || "")
    .split(/\n\n+/)
    .map((part) => part.trim())
    .filter(Boolean)
    .forEach((part) => {
      parts.push(`<p>${escapeHtml(part).replaceAll("\n", "<br>")}</p>`);
    });
  return parts.join("");
}

export function asAlbums(entries) {
  const albums = new Map();
  entries.forEach((entry) => {
    const id = String(entry.extra?.albumId || entry.id);
    const title = entry.extra?.albumTitle || entry.title || id;
    const album = albums.get(id) || {
      id,
      title,
      eyebrow: "站长上传",
      description: "由站长控制器上传。",
      tone: "from-[#F6FBFF] via-[#FFF8F1] to-[#FFEAF4]",
      cover: entry.imageUrl,
      images: [],
    };
    if (entry.imageUrl) album.images.push(entry.imageUrl);
    if (!album.cover) album.cover = entry.imageUrl;
    albums.set(id, album);
  });
  return [...albums.values()];
}

export function asMoments(entries) {
  return entries.map((entry) => ({
    id: entry.id,
    year: String(entry.extra?.year || entry.createdAt?.slice(0, 4) || ""),
    date: String(entry.extra?.date || ""),
    type: entry.extra?.type || entry.title,
    tone: "aurora",
    lines: String(entry.body || "").split("\n").filter(Boolean),
  }));
}

export function asVideos(entries) {
  return entries.map((entry) => ({
    id: entry.id,
    bvid: entry.extra?.bvid || "",
    title: entry.title,
    description: entry.body,
    cover: entry.imageUrl,
    duration: entry.extra?.duration || "",
    publishedAt: entry.extra?.publishedAt || entry.createdAt,
  }));
}

export function asPosts(entries) {
  return entries.map((entry) => ({
    id: entry.id,
    slug: String(entry.id),
    title: entry.title,
    date: String(entry.createdAt || "").slice(0, 10),
    cover: entry.imageUrl,
    description: String(entry.body || "").replace(/\s+/g, " ").slice(0, 120),
    categories: [],
    tags: [],
    body: entry.body,
  }));
}

export function asJournal(entries) {
  return entries.map((entry) => {
    const words = [...String(entry.body || "")].filter((ch) => !/\s/.test(ch)).length;
    const excerpt = String(entry.body || "").slice(0, 80);
    return {
      id: entry.id,
      slug: String(entry.id),
      date: String(entry.createdAt || "").slice(0, 10),
      title_zh: entry.title,
      title_en: entry.title,
      excerpt_zh: excerpt,
      excerpt_en: excerpt,
      words: String(words),
      reads: "0",
      minutes: String(Math.max(1, Math.ceil(words / 400))),
      bodyHtmlZh: bodyHtml(entry.body, entry.imageUrl),
      bodyHtmlEn: bodyHtml(entry.body, entry.imageUrl),
    };
  });
}

export function asProjects(entries) {
  return entries.map((entry) => {
    const url = entry.extra?.url || "https://www.miku.coffee/";
    const tags = String(entry.extra?.tags || "星空馆")
      .split(/[,，]/)
      .map((tag) => tag.trim())
      .filter(Boolean);
    return {
      id: String(entry.id),
      title: entry.title,
      summary: entry.body,
      author: "Spume",
      category: "项目",
      tags: tags.length ? tags : ["星空馆"],
      cover: entry.imageUrl,
      preview: entry.imageUrl,
      url,
      source: url,
      docs: url,
      reviewedAt: String(entry.createdAt || "").slice(0, 10),
      license: "站长发布",
      licenseUrl: url,
      isGuide: false,
      demo: { kind: "link", note: entry.body, label: "打开项目", url },
      sections: [["项目介绍", entry.body, ""]],
    };
  });
}
