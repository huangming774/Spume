#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""从 content/ 下的 Markdown 生成文章列表、详情页与说说页。运行: python build_content.py"""

from __future__ import annotations

import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ARTICLES_DIR = ROOT / "content" / "articles"
POST_DIR = ROOT / "post"
REACT_DATA = ROOT / "src" / "data" / "content.json"
HOME_CSS = ROOT / "src" / "styles" / "home.css"
HELLO_FONT = ROOT / "src" / "assets" / "hello-font.b64"

try:
    import markdown as md_lib

    def md_to_html(src: str) -> str:
        return md_lib.markdown(
            src,
            extensions=["extra", "sane_lists", "nl2br"],
            output_format="html5",
        )

except ImportError:

    def md_to_html(src: str) -> str:
        return _md_to_html_fallback(src)


NAV = """    <a href="{p}index.html" class="nav-item{home}"><span class="nav-icon">HOME</span><span class="nav-label" data-i18n="navHome">首页</span></a>
    <a href="{p}archives.html" class="nav-item{arc}"><span class="nav-icon">ARC</span><span class="nav-label" data-i18n="navArchive">归档</span></a>
    <a href="{p}articles.html" class="nav-item{art}"><span class="nav-icon">ART</span><span class="nav-label" data-i18n="navArticle">文章</span></a>
"""

PAGE_STYLES = """
  * { margin:0; padding:0; box-sizing:border-box; }
  body {
    min-height:100vh; display:flex; overflow-x:hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', sans-serif;
    background: #f9fafb;
  }
  .bg {
    position:fixed; inset:0; z-index:0;
    background:
      radial-gradient(ellipse 72% 58% at 18% 22%, rgba(186, 206, 255, 0.22), transparent 68%),
      radial-gradient(ellipse 62% 56% at 88% 78%, rgba(255, 218, 225, 0.18), transparent 64%),
      linear-gradient(165deg, #fdfefe 0%, #f8f9fb 40%, #fafbfc 100%);
    animation: gradMorph 16s ease-in-out infinite alternate;
  }
  @keyframes gradMorph { 0%,100%{opacity:1} 50%{opacity:.92} }
  .sidebar {
    position:relative; z-index:10; width:220px; min-width:220px; height:100vh;
    background: rgba(255,255,255,0.7);
    backdrop-filter: blur(22px); -webkit-backdrop-filter: blur(22px);
    border-right: 1px solid rgba(0,0,0,0.04);
    display:flex; flex-direction:column; padding:38px 22px; flex-shrink:0;
  }
  .sidebar-header { display:flex; align-items:center; gap:11px; margin-bottom:18px; }
  .avatar { width:36px; height:36px; border-radius:50%; overflow:hidden; border:1.5px solid rgba(0,0,0,0.1); }
  .avatar img { width:100%; height:100%; object-fit:cover; }
  .blog-name { color:#3a3a4a; font-size:13px; font-weight:500; }
  .sidebar-social { display:flex; gap:12px; margin-bottom:32px; }
  .sidebar-social a { width:28px; height:28px; display:flex; align-items:center; justify-content:center; border-radius:6px; }
  .sidebar-nav { flex:1; display:flex; flex-direction:column; }
  .nav-item {
    display:flex; align-items:center; gap:11px; padding:9px 13px; border-radius:7px;
    color:#b0b0b8; font-size:12px; text-decoration:none;
  }
  .nav-item:hover { background:rgba(0,0,0,0.02); color:#666; }
  .nav-item.active { background:rgba(0,0,0,0.03); color:#3a3a4a; }
  .nav-icon { width:17px; opacity:0.35; font-size:12px; }
  .nav-item.active .nav-icon { opacity:0.65; }
  .blog-name-full { font-size:12px; max-width:140px; line-height:1.35; }
  .sidebar-footer { padding-top:16px; border-top:1px solid rgba(0,0,0,0.03); color:#c0c0c4; font-size:9px; }
  .main { position:relative; z-index:5; flex:1; overflow-y:auto; height:100vh; padding:40px 50px; }
  .sidebar-toggle { position:absolute; top:12px; right:12px; width:28px; height:28px; border:1px solid rgba(0,0,0,0.1); background:rgba(255,255,255,0.6); border-radius:6px; cursor:pointer; font-size:14px; color:#999; display:flex; align-items:center; justify-content:center; z-index:20; transition:all 0.2s; }
  .sidebar-toggle:hover { background:rgba(0,0,0,0.04); color:#555; }
  .sidebar.collapsed { width:50px; min-width:50px; padding:20px 8px; }
  .sidebar.collapsed .sidebar-header,.sidebar.collapsed .blog-name,.sidebar.collapsed .nav-label,.sidebar.collapsed .sidebar-footer,.sidebar.collapsed .sidebar-social,.sidebar.collapsed .lang-toggle-wrap { display:none; }
  .sidebar.collapsed .nav-item { justify-content:center; padding:8px; }
  .sidebar.collapsed .sidebar-toggle { top:6px; right:auto; left:50%; transform:translateX(-50%); }
  .sidebar.collapsed .sidebar-nav { padding-top:30px; }
  @media (max-width:768px) {
    .sidebar { width:50px; min-width:50px; padding:20px 8px; }
    .blog-name,.nav-label,.sidebar-footer,.sidebar-social,.sidebar-toggle,.lang-toggle-wrap { display:none; }
    .nav-item { justify-content:center; padding:6px; }
    .main { padding:24px 16px; }
    .post-body.prose table { display:block; overflow-x:auto; font-size:12px; }
    .post-body.prose th,.post-body.prose td { padding:6px 8px; }
    .post-body.prose pre { margin:14px 0; font-size:12px; }
    .post-body.prose pre code { font-size:11px; padding:10px 12px; }
    .post-body.prose pre::before { height:28px; }
    .post-body.prose pre::after { top:8px; left:10px; width:9px; height:9px; box-shadow:20px 0 0 #febc2e,40px 0 0 #28c840; }
    .code-copy-btn { top:4px; right:6px; font-size:10px; padding:1px 8px; }
    .post-body.prose .quote-art { padding:18px 16px; font-size:15px; }
    .article-card { padding:16px; }
    .article-card h2 { font-size:15px; }
    .page-title { font-size:22px; }
    .post-title { font-size:20px; }
    .album-grid,.photo-grid { grid-template-columns:repeat(2,1fr); gap:8px; }
    .gallery-feed { gap:20px; }
  }
  @media (max-width:480px) {
    .main { padding:16px 12px; }
    .post-body.prose table { font-size:11px; }
    .post-body.prose th,.post-body.prose td { padding:4px 6px; }
    .post-body.prose .quote-art { padding:14px 12px; font-size:14px; }
    .article-card { padding:12px; }
    .album-grid,.photo-grid { grid-template-columns:repeat(2,1fr); gap:6px; }
  }
"""

SIDEBAR_SOCIAL = """
    <a href="https://space.bilibili.com/1061280173" target="_blank" rel="noopener noreferrer" title="B站"><svg width="18" height="18" viewBox="0 0 24 24" fill="none"><rect x="3.5" y="5" width="17" height="12.5" rx="2" stroke="#aaa" stroke-width="1.2"/><path d="M6 2l2 3M18 2l-2 3" stroke="#aaa" stroke-width="1.2" stroke-linecap="round"/><path d="M8 9v5M11 9v5M14 9v5M16 9v5" stroke="#aaa" stroke-width="1" stroke-linecap="round"/></svg></a>
    <a href="https://github.com/bistutzyy" target="_blank" rel="noopener noreferrer" title="GitHub"><svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M12 2C6.48 2 2 6.48 2 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.23.66-.5v-1.69c-2.78.6-3.36-1.34-3.36-1.34-.46-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.9 1.52 2.34 1.08 2.91.83.1-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.92 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.65 0 0 .84-.27 2.75 1.02A9.6 9.6 0 0112 6.8c.85.004 1.7.11 2.5.34 1.91-1.29 2.75-1.02 2.75-1.02.55 1.38.2 2.4.1 2.65.64.7 1.03 1.59 1.03 2.68 0 3.82-2.34 4.66-4.57 4.91.36.31.68.92.68 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12c0-5.52-4.48-10-10-10z" fill="#aaa"/></svg></a>
"""

WEATHER_HEAD = """
<link rel="stylesheet" href="{prefix}site-weather.css">
<script>try{{if(localStorage.getItem('siteWeather')==='rainy')document.documentElement.classList.add('weather-rainy');}}catch(e){{}}</script>
<script src="{prefix}site-weather.js" defer></script>"""

WEATHER_BODY = """
<div class="weather-mist" aria-hidden="true"></div>
<canvas id="rainCanvas" class="rain-canvas" aria-hidden="true"></canvas>"""

WEATHER_BTN = """    <button type="button" class="weather-toggle" id="weatherToggle" aria-pressed="false"><span class="weather-toggle-icon" aria-hidden="true">☔</span><span class="weather-toggle-label">雨天</span></button>"""


def esc(s) -> str:
    return html.escape(str(s or ""), quote=True)


def i18n_pair(zh: str, en: str) -> str:
    return (
        f'<span class="i18n-zh">{esc(zh)}</span>'
        f'<span class="i18n-en" style="display:none">{esc(en)}</span>'
    )


def i18n_body_br(zh: str, en: str) -> str:
    zh_h = esc(zh).replace("\n", "<br>")
    en_h = esc(en).replace("\n", "<br>")
    return f'<span class="i18n-zh">{zh_h}</span><span class="i18n-en" style="display:none">{en_h}</span>'


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---"):
        return {}, text
    m = re.match(r"^---\r?\n(.*?)\r?\n---\r?\n?", text, re.DOTALL)
    if not m:
        return {}, text
    meta: dict[str, str] = {}
    key = None
    buf: list[str] = []
    for line in m.group(1).splitlines():
        if re.match(r"^[\w-]+:\s*", line):
            if key is not None:
                meta[key] = "\n".join(buf).strip()
            key, rest = re.split(r":\s*", line, maxsplit=1)
            key = key.strip()
            rest = rest.strip()
            if rest in ("|", ">"):
                buf = []
            elif rest:
                meta[key] = rest.strip("\"'")
                key = None
                buf = []
            else:
                buf = []
        elif key is not None:
            buf.append(line)
    if key is not None:
        meta[key] = "\n".join(buf).strip()
    return meta, text[m.end() :]


def split_bilingual(body: str) -> tuple[str, str]:
    parts = re.split(r"\n---en---\n", body, maxsplit=1)
    zh = parts[0].strip()
    en = parts[1].strip() if len(parts) > 1 else zh
    return zh, en


def inline_md(text: str) -> str:
    # Preserve raw HTML tags before escaping
    html_tags: list[str] = []
    def _save_html(m):
        html_tags.append(m.group(0))
        return f"\x00HTML{len(html_tags)-1}\x00"
    text = re.sub(r"<[^>]+>", _save_html, text)
    text = esc(text)
    text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    text = re.sub(r"\*([^*]+)\*", r"<em>\1</em>", text)
    text = re.sub(
        r"\[([^\]]+)\]\(([^)]+)\)",
        r'<a href="\2" target="_blank" rel="noopener noreferrer">\1</a>',
        text,
    )
    # Restore raw HTML
    for i, tag in enumerate(html_tags):
        text = text.replace(f"\x00HTML{i}\x00", tag)
    return text


def _md_to_html_fallback(src: str) -> str:
    if not src.strip():
        return ""
    lines = src.replace("\r\n", "\n").split("\n")
    out: list[str] = []
    i = 0
    in_code = False
    code_buf: list[str] = []
    list_type: str | None = None
    para_buf: list[str] = []

    def close_list():
        nonlocal list_type
        if list_type:
            out.append(f"</{list_type}>")
            list_type = None

    def flush_para():
        if para_buf:
            out.append(f"<p>{inline_md(' '.join(para_buf))}</p>")
            para_buf.clear()

    def is_sep_row(s: str) -> bool:
        """Detect |---|---| style separator row."""
        return bool(re.match(r"^\|\s*[-:]+\s*(\|\s*[-:]+\s*)+\|?\s*$", s))

    def has_table_row(s: str) -> bool:
        """Detect | col1 | col2 | style row."""
        return s.strip().startswith("|") and s.strip().endswith("|")

    while i < len(lines):
        line = lines[i]
        # Table detection: look ahead for separator row
        if (
            has_table_row(line)
            and not is_sep_row(line)
            and i + 2 < len(lines)
            and is_sep_row(lines[i + 1])
        ):
            flush_para()
            close_list()
            # Build header
            headers = [c.strip() for c in line.strip().strip("|").split("|")]
            i += 1  # skip header row
            i += 1  # skip separator row
            rows: list[list[str]] = []
            while i < len(lines) and has_table_row(lines[i]):
                rows.append([c.strip() for c in lines[i].strip().strip("|").split("|")])
                i += 1
            out.append("<table>")
            out.append("<thead><tr>" + "".join(f"<th>{inline_md(h)}</th>" for h in headers) + "</tr></thead>")
            out.append("<tbody>")
            for row in rows:
                out.append("<tr>" + "".join(f"<td>{inline_md(c)}</td>" for c in row) + "</tr>")
            out.append("</tbody></table>")
            continue
        if line.strip().startswith("```"):
            flush_para()
            close_list()
            if not in_code:
                in_code = True
                code_buf = []
            else:
                out.append("<pre><code>" + esc("\n".join(code_buf)) + "</code></pre>")
                in_code = False
            i += 1
            continue
        if in_code:
            code_buf.append(line)
            i += 1
            continue
        if re.match(r"^#{1,3}\s+", line):
            flush_para()
            close_list()
            level = min(len(line) - len(line.lstrip("#")), 3)
            out.append(f"<h{level}>{inline_md(line.lstrip('#').strip())}</h{level}>")
            i += 1
            continue
        if re.match(r"^>\s?", line):
            flush_para()
            close_list()
            quotes = []
            while i < len(lines) and re.match(r"^>\s?", lines[i]):
                quotes.append(re.sub(r"^>\s?", "", lines[i]))
                i += 1
            out.append(f"<blockquote><p>{inline_md(' '.join(quotes))}</p></blockquote>")
            continue
        if re.match(r"^[-*]\s+", line):
            flush_para()
            if list_type != "ul":
                close_list()
                out.append("<ul>")
                list_type = "ul"
            out.append(f"<li>{inline_md(line[2:].strip())}</li>")
            i += 1
            continue
        if not line.strip():
            flush_para()
            close_list()
            i += 1
            continue
        para_buf.append(line.strip())
        i += 1
    flush_para()
    close_list()
    return "\n".join(out)


def nav_html(active: str, prefix: str = "") -> str:
    mapping = {
        "index": "home",
        "archives": "arc",
        "articles": "art",
        "post": "art",
    }
    act = mapping.get(active, "")
    fmt = {k: " active" if k == act else "" for k in ("home", "arc", "art")}
    fmt["p"] = prefix
    return NAV.format(**fmt)


def page_shell(title: str, active: str, inner: str, *, depth: int = 0, extra_scripts: str = "") -> str:
    prefix = "../" * depth
    fonts = ""
    if active in ("post",):
        fonts = (
            '<link rel="preconnect" href="https://fonts.googleapis.com">'
            '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>'
            '<link href="https://fonts.googleapis.com/css2?family=Ma+Shan+Zheng&family=ZCOOL+XiaoWei&display=swap" rel="stylesheet">'
        )
    hljs_cdn = ""
    copy_script = ""
    if active in ("post",):
        hljs_cdn = (
            '<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/atom-one-dark.min.css">'
            '<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"></script>'
        )
        copy_script = """<script>
(function(){
  var pres = document.querySelectorAll('.post-body.prose pre');
  pres.forEach(function(pre){
    var btn = document.createElement('button');
    btn.className = 'code-copy-btn';
    btn.textContent = 'Copy';
    btn.addEventListener('click', function(){
      var code = pre.querySelector('code');
      var text = code ? code.innerText : pre.innerText;
      navigator.clipboard.writeText(text).then(function(){
        btn.textContent = 'Copied!';
        setTimeout(function(){ btn.textContent = 'Copy'; }, 2000);
      });
    });
    pre.style.position = 'relative';
    pre.appendChild(btn);
  });
  hljs.highlightAll();
  pres.forEach(function(pre){
    var code = pre.querySelector('code');
    if (code) {
      var cls = code.className || '';
      // plaintext / no-language / text / output → mark as output
      if (/language-plaintext|language-text|language-output|language-none/.test(cls) || !/language-/.test(cls)) {
        pre.classList.add('code-output');
      }
    }
  });
})();
</script>"""
    return f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{esc(title)}</title>
<script src="{prefix}site-i18n.js" defer></script>
{WEATHER_HEAD.format(prefix=prefix)}
{fonts}
{hljs_cdn}
<link rel="stylesheet" href="{prefix}site-pages.css">
<style>{PAGE_STYLES}</style>
<link rel="stylesheet" href="{prefix}site-mobile-nav.css">
</head>
<body>
<div class="bg"></div>
{WEATHER_BODY}
<aside class="sidebar">
  <button type="button" class="sidebar-toggle" onclick="this.parentElement.classList.toggle('collapsed')" title="折叠导航栏">☰</button>
  <div class="sidebar-header">
    <div class="avatar"><img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/1.png" alt=""></div>
    <div><div class="blog-name blog-name-full" data-i18n="blogTitle">桃之夭夭の创作屋</div></div>
  </div>
  <div class="sidebar-social">{SIDEBAR_SOCIAL}</div>
  <nav class="sidebar-nav">
{nav_html(active, prefix)}
  </nav>
  <div class="lang-toggle-wrap">
    <button type="button" class="lang-toggle" id="siteLangToggle">English</button>
{WEATHER_BTN}
  </div>
  <div class="sidebar-footer" data-i18n="footerLine">桃之夭夭 © 2026</div>
</aside>
<main class="main">
{inner}
</main>
{extra_scripts}
{copy_script}
</body>
</html>
"""


def load_articles() -> list[dict]:
    items = []
    for path in sorted(ARTICLES_DIR.glob("*.md")):
        meta, body = parse_frontmatter(path.read_text(encoding="utf-8"))
        zh, en = split_bilingual(body)
        slug = meta.get("slug") or path.stem
        item = {
            "slug": slug,
            "date": meta.get("date", "1970-01-01"),
            "title_zh": meta.get("title_zh", slug),
            "title_en": meta.get("title_en", slug),
            "excerpt_zh": meta.get("excerpt_zh", ""),
            "excerpt_en": meta.get("excerpt_en", ""),
            "words": meta.get("words", ""),
            "reads": meta.get("reads", ""),
            "minutes": meta.get("minutes", ""),
            "body_zh": zh,
            "body_en": en,
        }
        mod = meta.get("modified", "").strip()
        if mod:
            item["modified"] = mod
        items.append(item)
    items.sort(key=lambda x: x["date"], reverse=True)
    return items

def article_card(a: dict) -> str:
    return f"""    <a href="post/{esc(a['slug'])}.html" class="article-card">
      <h2>{i18n_pair(a['title_zh'], a['title_en'])}</h2>
      <p class="excerpt">{i18n_pair(a['excerpt_zh'], a['excerpt_en'])}</p>
      <div class="article-meta">
        <span>{esc(a['date'])}</span>
        <span>{esc(a['words'])} <span data-i18n="metaWords">字</span></span>
        <span>{esc(a['reads'])} <span data-i18n="metaReads">阅读</span></span>
        <span><span data-i18n="metaMinutes">约</span>{esc(a['minutes'])} <span data-i18n="metaMinSuffix">分钟</span></span>
      </div>
    </a>"""

def build_articles_list(articles: list[dict]) -> None:
    cards = "\n".join(article_card(a) for a in articles)
    if not cards:
        cards = '    <p class="page-lead">暂无文章。在 content/articles/ 添加 .md 后运行 python build_content.py</p>'
    inner = f"""<div class="page-main-inner">
  <h1 class="page-title" data-i18n="articlesTitle">文章</h1>
  <div class="article-list">
{cards}
  </div>
</div>"""
    (ROOT / "articles.html").write_text(page_shell("文章 Articles", "articles", inner), encoding="utf-8")
    print(f"OK: articles.html ({len(articles)} 篇)")


def build_posts(articles: list[dict]) -> None:
    POST_DIR.mkdir(exist_ok=True)
    for a in articles:
        zh_html = md_to_html(a["body_zh"])
        en_html = md_to_html(a["body_en"])
        inner = f"""<article class="page-main-inner post-page">
  <a href="../articles.html" class="post-back">← <span data-i18n="navArticle">文章</span></a>
  <header class="post-header">
    <h1 class="post-title">{i18n_pair(a['title_zh'], a['title_en'])}</h1>
    <div class="post-meta article-meta">
      <span>{esc(a['date'])}</span>
      <span>{esc(a['words'])} <span data-i18n="metaWords">字</span></span>
      <span>{esc(a['reads'])} <span data-i18n="metaReads">阅读</span></span>
      <span><span data-i18n="metaMinutes">约</span>{esc(a['minutes'])} <span data-i18n="metaMinSuffix">分钟</span></span>
    </div>
  </header>
  <div class="post-body prose i18n-zh">{zh_html}</div>
  <div class="post-body prose i18n-en" style="display:none">{en_html}</div>
</article>"""
        inner = inner.replace("</div>", "</div>")
        (POST_DIR / f"{a['slug']}.html").write_text(
            page_shell(a["title_zh"], "post", inner, depth=1),
            encoding="utf-8",
        )
    print(f"OK: post/ ({len(articles)} 篇)")

def build_archives(articles: list[dict]) -> None:
    by_year: dict[str, list[dict]] = {}
    for a in articles:
        by_year.setdefault(a["date"][:4], []).append(a)

    skins = [
        "moment-skin--aurora", "moment-skin--sakura", "moment-skin--starry",
        "moment-skin--watercolor", "moment-skin--ticket", "moment-skin--envelope",
        "moment-skin--candy", "moment-skin--journal", "moment-skin--wave",
        "moment-skin--frost", "moment-skin--sunset", "moment-skin--mint",
        "moment-skin--neon", "moment-skin--glass", "moment-skin--stamp",
    ]
    idx = 0
    parts = []
    for year in sorted(by_year, reverse=True):
        parts.append(f'    <div class="timeline-year"><span>{year}</span></div>')
        for a in by_year[year]:
            skin = skins[idx % len(skins)]
            idx += 1
            md = a["date"][5:10] if len(a["date"]) >= 10 else a["date"]
            parts.append(
                f"""    <div class="timeline-item">
      <a href="post/{esc(a['slug'])}.html" class="timeline-card moment {skin}">
        <div class="row">
          <span class="timeline-date">{md}</span>
          <span class="timeline-title">{i18n_pair(a['title_zh'], a['title_en'])}</span>
          <span class="timeline-tag" data-i18n="tagDone">完成</span>
        </div>
      </a>
    </div>"""
            )
    timeline = "\n".join(parts) if parts else '    <p class="page-lead">暂无文章。</p>'
    new_block = f'  <div class="archive-timeline">\n{timeline}\n  </div>'
    inner = f'<div class="page-main-inner page-art page-art--neat">\n  <h1 class="page-title page-title--art" data-i18n="archiveTitle">归档</h1>\n  <p class="page-lead" data-i18n="archiveLead">时光轴</p>\n{new_block}\n</div>\n'
    (ROOT / "archives.html").write_text(page_shell("归档 Archives", "archives", inner), encoding="utf-8")
    print("OK: archives.html")


def build_react_data(articles: list[dict]) -> None:
    import json

    payload = {
        "articles": [
            {
                **{k: a[k] for k in (
                    "slug", "date", "title_zh", "title_en", "excerpt_zh", "excerpt_en",
                    "words", "reads", "minutes",
                )},
                "bodyHtmlZh": md_to_html(a["body_zh"]),
                "bodyHtmlEn": md_to_html(a["body_en"]),
            }
            for a in articles
        ],
    }
    REACT_DATA.parent.mkdir(parents=True, exist_ok=True)
    REACT_DATA.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"OK: {REACT_DATA.relative_to(ROOT)} ({len(articles)} 篇)")


def _extract_style(html_path: Path, out_path: Path) -> None:
    if not html_path.is_file():
        return
    text = html_path.read_text(encoding="utf-8")
    m = re.search(r"<style>(.*?)</style>", text, re.DOTALL)
    if not m:
        return
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(m.group(1).strip() + "\n", encoding="utf-8")
    print(f"OK: {out_path.relative_to(ROOT)}")


def _home_html_source() -> Path | None:
    for name in ("legacy-home-source.html", "legacy/index.html"):
        p = ROOT / name
        if p.is_file() and p.stat().st_size > 5000:
            return p
    vite_entry = ROOT / "index.html"
    if vite_entry.is_file() and vite_entry.stat().st_size > 5000:
        return vite_entry
    return None


def extract_static_assets() -> None:
    index = _home_html_source()
    if index:
        _extract_style(index, HOME_CSS)
    elif (ROOT / "message.html").is_file():
        _extract_style(ROOT / "message.html", HOME_CSS)
        print("提示: home.css 暂用 message.html 样式；完整首页请放置 legacy-home-source.html")
    else:
        HOME_CSS.parent.mkdir(parents=True, exist_ok=True)
        HOME_CSS.write_text("/* home */\n", encoding="utf-8")
    if index:
        text = index.read_text(encoding="utf-8")
        m = re.search(r'const FONT_B64 = "([^"]+)"', text)
        if m:
            HELLO_FONT.parent.mkdir(parents=True, exist_ok=True)
            HELLO_FONT.write_text(m.group(1), encoding="utf-8")
            print(f"OK: {HELLO_FONT.relative_to(ROOT)}")


def main() -> None:
    ARTICLES_DIR.mkdir(parents=True, exist_ok=True)
    articles = load_articles()
    build_react_data(articles)
    extract_static_assets()
    print(f"完成: {len(articles)} 篇文章")
    print("更新 content/ 后请运行: python build_content.py")


if __name__ == "__main__":
    main()
