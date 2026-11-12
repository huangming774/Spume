import { parseFrontMatter, postExcerpt } from "./renderPost.js";

const files = import.meta.glob("../../../../blog/source/_posts/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
});

function asList(value) {
  if (Array.isArray(value)) return value;
  if (!value) return [];
  return String(value).split(/[,，]/).map((item) => item.trim()).filter(Boolean);
}

export const posts = Object.entries(files)
  .map(([path, raw]) => {
    const slug = decodeURIComponent(path.split("/").pop().replace(/\.md$/, ""));
    const { meta, body } = parseFrontMatter(raw);
    return {
      slug,
      title: meta.title || slug,
      date: String(meta.date || ""),
      cover: meta.cover || "",
      description: meta.description || postExcerpt(body),
      categories: asList(meta.categories),
      tags: asList(meta.tags),
      body,
    };
  })
  .sort((a, b) => (a.date < b.date ? 1 : -1));

export function findPost(splat) {
  const parts = decodeURIComponent(splat || "").split("/").filter(Boolean);
  const slug = (parts.at(-1) || "").replace(/\.html$/, "");
  return posts.find((post) => post.slug === slug);
}

export function postPath(slug) {
  return `/blog/${encodeURIComponent(slug)}`;
}
