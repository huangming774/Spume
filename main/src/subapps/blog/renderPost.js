import { marked } from "marked";

marked.use({ gfm: true, breaks: false });

const CODE_TOKEN = "\u0000CODE";

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeAttr(value) {
  return escapeHtml(value).replace(/'/g, "&#39;");
}

function safeColor(color) {
  const name = String(color || "default").toLowerCase();
  return /^[a-z0-9-]{1,24}$/.test(name) ? name : "default";
}

function protectCode(markdown) {
  const blocks = [];
  const text = markdown.replace(/```[\s\S]*?```|~~~[\s\S]*?~~~|`[^`\n]+`/g, (block) => {
    const token = `${CODE_TOKEN}${blocks.length}\u0000`;
    blocks.push(block);
    return token;
  });
  return { text, blocks };
}

function restoreCode(markdown, blocks) {
  return markdown.replace(new RegExp(`${CODE_TOKEN}(\\d+)\u0000`, "g"), (_, index) => blocks[Number(index)] ?? "");
}

function parseFragment(fragment, blocks) {
  const source = restoreCode(fragment, blocks).trim();
  if (!source) return "";
  return marked.parse(source, { async: false });
}

function replaceInline(markdown) {
  return markdown
    .replace(/\{%\s*span\s+([A-Za-z0-9_-]+)\s*,\s*([\s\S]*?)\s*%\}/g, (_, color, text) => {
      const label = String(text).trim().replace(/^["'“”]+|["'“”]+$/g, "");
      return `<span class="bt-span bt-${safeColor(color)}">${escapeHtml(label)}</span>`;
    })
    .replace(/\{%\s*label\s+(.+?)\s+([A-Za-z]+)\s*%\}/g, (_, text, color) => {
      return `<span class="bt-label bt-${safeColor(color)}">${escapeHtml(text.trim())}</span>`;
    })
    .replace(/\{%\s*checkbox(?:\s+(checked))?(?:\s+([A-Za-z]+))?\s*,\s*([\s\S]*?)\s*%\}/g, (_, checked, color, text) => {
      return `<span class="bt-check bt-${safeColor(color)}"><input type="checkbox" disabled${checked ? " checked" : ""} /> ${escapeHtml(text.trim())}</span>`;
    })
    .replace(/\{%\s*progress\s+(\d+%)\s*,\s*([A-Za-z]+)\s*,\s*([\s\S]*?)\s*%\}/g, (_, percent, color, text) => {
      const width = Math.max(0, Math.min(100, Number.parseInt(percent, 10) || 0));
      return `<span class="bt-progress bt-${safeColor(color)}"><i style="--bt-progress:${width}%"></i><b>${escapeHtml(text.trim())}</b></span>`;
    })
    .replace(/\{%\s*image\s+(\S+?)\s*,\s*([\s\S]*?)\s*%\}/g, (_, url, rest) => {
      const alt = rest.match(/alt=([^,]+)/)?.[1]?.trim() ?? "";
      const width = rest.match(/width=([^,\s]+)/)?.[1]?.trim();
      const style = width ? ` style="width:${escapeAttr(width)}"` : "";
      return `<img src="${escapeAttr(url)}" alt="${escapeAttr(alt)}"${style} />`;
    })
    .replace(/\{%\s*video\s+(\S+?)\s*%\}/g, (_, url) => {
      return `<video class="bt-video" controls src="${escapeAttr(url)}"></video>`;
    })
    .replace(/\{%\s*u\s+([\s\S]*?)\s*%\}/g, (_, text) => `<u>${escapeHtml(text.trim())}</u>`)
    .replace(/\{%\s*kbd\s+([\s\S]*?)\s*%\}/g, (_, text) => `<kbd>${escapeHtml(text.trim())}</kbd>`)
    .replace(/\{%\s*link\s+([^,]+?)\s*,\s*(https?:\/\/\S+?)\s*,\s*(https?:\/\/\S+?)\s*%\}/g, (_, text, url, avatar) => {
      return `<a class="bt-link" href="${escapeAttr(url.replace(/,$/, ""))}"><img src="${escapeAttr(avatar)}" alt="" />${escapeHtml(text.trim())}</a>`;
    })
    .replace(/\{%\s*ghcard\s+([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)[^%]*%\}/g, (_, repo) => {
      return `<a class="bt-link" href="${escapeAttr(`https://github.com/${repo}`)}">${escapeHtml(repo)}</a>`;
    });
}

function replaceBlock(markdown, pattern, render) {
  return markdown.replace(pattern, (...args) => `\n\n${render(...args)}\n\n`);
}

function renderSections(body, marker, className, blocks) {
  const chunks = body.split(new RegExp(`<!--\\s*${marker}\\s+`)).slice(1);
  if (!chunks.length) return parseFragment(body, blocks);
  return `<div class="${className}">${chunks.map((chunk) => {
    const titleEnd = chunk.indexOf("-->");
    const title = titleEnd >= 0 ? chunk.slice(0, titleEnd).trim() : "";
    let content = titleEnd >= 0 ? chunk.slice(titleEnd + 3) : chunk;
    content = content.replace(new RegExp(`<!--\\s*end${marker}\\s*-->`, "g"), "");
    return `<section><h3>${escapeHtml(title)}</h3>${parseFragment(content, blocks)}</section>`;
  }).join("")}</div>`;
}

function renderBlocks(markdown, blocks) {
  let text = markdown;
  const passes = [
    [/\{%\s*note(?:\s+([A-Za-z]+))?\s*%\}([\s\S]*?)\{%\s*endnote\s*%\}/g, (_, color, body) => `<aside class="bt-note bt-${safeColor(color)}">${parseFragment(body, blocks)}</aside>`],
    [/\{%\s*tip(?:\s+([A-Za-z]+))?\s*%\}([\s\S]*?)\{%\s*endtip\s*%\}/g, (_, color, body) => `<aside class="bt-note bt-tip bt-${safeColor(color)}">${parseFragment(body, blocks)}</aside>`],
    [/\{%\s*poem\s+([^,]+?)\s*,\s*([^%]+?)\s*%\}([\s\S]*?)\{%\s*endpoem\s*%\}/g, (_, title, author, body) => `<figure class="bt-poem"><figcaption>${escapeHtml(title.trim())} · ${escapeHtml(author.trim())}</figcaption><pre>${escapeHtml(body.trim())}</pre></figure>`],
    [/\{%\s*blockquote(?:\s+([^%]+?))?\s*%\}([\s\S]*?)\{%\s*endblockquote\s*%\}/g, (_, author, body) => `<blockquote class="bt-quote">${parseFragment(body, blocks)}${author ? `<footer>${escapeHtml(author.trim())}</footer>` : ""}</blockquote>`],
    [/\{%\s*mermaid\s*%\}([\s\S]*?)\{%\s*endmermaid\s*%\}/g, (_, body) => `<pre class="bt-mermaid"><code>${escapeHtml(body.trim())}</code></pre>`],
    [/\{%\s*gallery\s*%\}([\s\S]*?)\{%\s*endgallery\s*%\}/g, (_, body) => `<div class="bt-gallery">${parseFragment(body, blocks)}</div>`],
    [/\{%\s*carousel\b[^%]*%\}([\s\S]*?)\{%\s*endcarousel\s*%\}/g, (_, body) => `<div class="bt-gallery">${parseFragment(body, blocks)}</div>`],
    [/\{%\s*timeline\s+([^,%]+?)(?:\s*,\s*([A-Za-z]+))?\s*%\}([\s\S]*?)\{%\s*endtimeline\s*%\}/g, (_, title, color, body) => `<section class="bt-timeline bt-${safeColor(color)}"><h3>${escapeHtml(title.trim())}</h3>${renderSections(body, "timeline", "bt-timeline-list", blocks)}</section>`],
    [/\{%\s*tabs\b[^%]*%\}([\s\S]*?)\{%\s*endtabs\s*%\}/g, (_, body) => renderSections(body, "tab", "bt-tabs", blocks)],
    [/\{%\s*folding\s+[A-Za-z]+\s*,\s*([^%]+?)\s*%\}([\s\S]*?)\{%\s*endfolding\s*%\}/g, (_, title, body) => `<details class="bt-folding"><summary>${escapeHtml(title.trim())}</summary>${parseFragment(body, blocks)}</details>`],
  ];
  for (let round = 0; round < 6; round += 1) {
    const next = passes.reduce((current, [pattern, render]) => replaceBlock(current, pattern, render), text);
    if (next === text) break;
    text = next;
  }
  return text;
}

function stripLeftoverTags(markdown) {
  return markdown.replace(/\{%\s*([\s\S]*?)\s*%\}/g, (_, inner) => {
    const url = inner.match(/https?:\/\/[^\s%]+/)?.[0];
    if (url) return url;
    return "";
  });
}

export function renderPostBody(markdown) {
  const { text, blocks } = protectCode(String(markdown || ""));
  const rendered = stripLeftoverTags(renderBlocks(replaceInline(text), blocks));
  return marked.parse(restoreCode(rendered, blocks), { async: false });
}

export function parseFrontMatter(raw) {
  const match = String(raw).match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/);
  const meta = {};
  if (!match) return { meta, body: String(raw) };
  for (const line of match[1].split(/\r?\n/)) {
    const index = line.indexOf(":");
    if (index <= 0) continue;
    const key = line.slice(0, index).trim();
    let value = line.slice(index + 1).trim();
    if (value.startsWith("[") && value.endsWith("]")) {
      meta[key] = value.slice(1, -1).split(",").map((item) => item.trim()).filter(Boolean);
    } else {
      value = value.replace(/^['"]|['"]$/g, "");
      meta[key] = value;
    }
  }
  return { meta, body: raw.slice(match[0].length) };
}

export function postExcerpt(body) {
  const plain = String(body)
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/\{%[\s\S]*?%\}/g, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/[#>*_[\]()`!-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return plain.length > 96 ? `${plain.slice(0, 96)}…` : plain;
}
