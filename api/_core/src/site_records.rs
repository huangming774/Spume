//! 站长控制台增删六块内容时改写仓库里的数据文件。
//!
//! 文章和建站日志是 Markdown / JSON。关于我、相册、碎语、哔哩和星空馆是 JS 数组。
//! 这里只产生新的文件文本，提交由 `github_publish` 完成。

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::js_source::SourceError;

/// 博客文章在仓库里的路径。只接受 `_posts` 下的单个 Markdown 文件。
pub fn blog_post_path(path: &str) -> Result<&str, SourceError> {
    let path = path.trim();
    if path.contains("..") || path.contains('\\') || path.contains('\n') || path.contains('\0') {
        return Err(SourceError::Invalid("文章路径无效"));
    }
    let name = path
        .strip_prefix("blog/source/_posts/")
        .ok_or(SourceError::Invalid("文章路径无效"))?;
    if name.is_empty() || name.contains('/') || !name.ends_with(".md") || name == ".md" {
        return Err(SourceError::Invalid("文章路径无效"));
    }
    Ok(path)
}

/// 建站日志的文件名。中文标题用时间戳，避免路径里出现空格。
pub fn journal_slug(title: &str, now: DateTime<Utc>) -> String {
    let stamp = now.format("%Y%m%d%H%M%S");
    let ascii: String = title
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .take(40)
        .collect();
    if ascii.is_empty() {
        format!("journal-{stamp}")
    } else {
        format!("{ascii}-{stamp}")
    }
}

/// 建站日志 slug 只能是字母、数字和短横线。
pub fn journal_slug_ok(slug: &str) -> bool {
    let mut chars = slug.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphanumeric()
        && slug.len() <= 80
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

/// 写进 `build/content/articles` 的 Markdown。中英正文先用同一份。
pub fn journal_markdown(article: &Value) -> String {
    let field = |key| article.get(key).and_then(Value::as_str).unwrap_or("");
    format!(
        "---\nslug: {slug}\ndate: {date}\ntitle_zh: {title_zh}\ntitle_en: {title_en}\nexcerpt_zh: {excerpt_zh}\nexcerpt_en: {excerpt_en}\nwords: {words}\nreads: \"0\"\nminutes: {minutes}\n---\n\n{body}\n\n---en---\n\n{body}\n",
        slug = yaml_quote(field("slug")),
        date = yaml_quote(field("date")),
        title_zh = yaml_quote(field("title_zh")),
        title_en = yaml_quote(field("title_en")),
        excerpt_zh = yaml_quote(field("excerpt_zh")),
        excerpt_en = yaml_quote(field("excerpt_en")),
        words = yaml_quote(field("words")),
        minutes = yaml_quote(field("minutes")),
        body = field("body"),
    )
}

/// 建站日志条目。封面地址会写成正文里的图片，图片本身已经在 S3。
pub fn journal_article(
    slug: &str,
    title: &str,
    body: &str,
    cover: &str,
    now: DateTime<Utc>,
) -> Result<Value, SourceError> {
    let title = title.trim();
    let body = body.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(SourceError::Invalid("发布标题需要 1 到 120 个字"));
    }
    if body.is_empty() || body.len() > 200_000 {
        return Err(SourceError::Invalid("发布正文不能为空，且不能过大"));
    }
    if !cover.is_empty() && !public_http_url(cover) {
        return Err(SourceError::Invalid(
            "封面 URL 必须是公开的 http 或 https 地址",
        ));
    }
    let site = now + chrono::Duration::hours(8);
    let words = body.chars().filter(|ch| !ch.is_whitespace()).count();
    let minutes = (words / 400).max(1);
    let excerpt: String = body.chars().take(80).collect();
    Ok(json!({
        "slug": slug,
        "date": site.format("%Y-%m-%d").to_string(),
        "title_zh": title,
        "title_en": title,
        "excerpt_zh": excerpt,
        "excerpt_en": excerpt,
        "words": words.to_string(),
        "reads": "0",
        "minutes": minutes.to_string(),
        "bodyHtmlZh": plain_html(body, cover),
        "bodyHtmlEn": plain_html(body, cover),
        "body": body,
    }))
}

/// 把一篇建站日志插到 `articles` 最前面。相同 slug 改为替换。
pub fn upsert_journal(source: &str, article: &Value) -> Result<String, SourceError> {
    let slug = article
        .get("slug")
        .and_then(Value::as_str)
        .ok_or(SourceError::Invalid("建站日志缺少 slug"))?;
    let mut root = parse_journal(source)?;
    let articles = journal_articles(&mut root)?;
    articles.retain(|item| item.get("slug").and_then(Value::as_str) != Some(slug));
    let mut stored = article.clone();
    if let Some(item) = stored.as_object_mut() {
        item.remove("body");
    }
    articles.insert(0, stored);
    pretty_journal(&root)
}

/// 按 slug 从建站日志 JSON 里删掉一篇。
pub fn remove_journal(source: &str, slug: &str) -> Result<String, SourceError> {
    if !journal_slug_ok(slug) {
        return Err(SourceError::Invalid("建站日志路径无效"));
    }
    let mut root = parse_journal(source)?;
    let articles = journal_articles(&mut root)?;
    let before = articles.len();
    articles.retain(|item| item.get("slug").and_then(Value::as_str) != Some(slug));
    if articles.len() == before {
        return Err(SourceError::Invalid("没有这篇建站日志"));
    }
    pretty_journal(&root)
}

/// 关于我的一条内容。
pub struct AboutDraft<'a> {
    /// 稳定 id。
    pub id: &'a str,
    /// 标题。
    pub title: &'a str,
    /// 正文。
    pub text: &'a str,
    /// 图片地址。可以空。
    pub image: &'a str,
}

/// 往关于我数组末尾加一条。
pub fn insert_about(source: &str, draft: AboutDraft<'_>) -> Result<String, SourceError> {
    let title = draft.title.trim();
    let text = draft.text.trim();
    let image = draft.image.trim();
    if !record_id(draft.id) {
        return Err(SourceError::Invalid("关于我条目无效"));
    }
    if title.is_empty() || text.is_empty() {
        return Err(SourceError::Invalid("关于我需要标题和正文"));
    }
    if !image.is_empty() && !public_http_url(image) {
        return Err(SourceError::Invalid(
            "图片 URL 必须是公开的 http 或 https 地址",
        ));
    }
    let card = format!(
        "  {{\n    id: {},\n    title: {},\n    text: {},\n    image: {},\n  }},\n",
        js_string(draft.id),
        js_string(title),
        js_string(text),
        js_string(image)
    );
    insert_card(source, "export const aboutEntries = [", &card)
}

/// 按 id 删除一条关于我。
pub fn remove_about(source: &str, id: &str) -> Result<String, SourceError> {
    remove_record(
        source,
        "export const aboutEntries = [",
        "id",
        id,
        "没有这条关于我",
    )
}

/// 站长手动收录的一条哔哩视频。
pub struct BiliDraft<'a> {
    /// BV 号。
    pub bvid: &'a str,
    /// 标题。
    pub title: &'a str,
    /// 简介。
    pub description: &'a str,
    /// 封面地址。可以空。
    pub cover: &'a str,
    /// 时长，例如 `03:21`。
    pub duration: &'a str,
    /// 发布日期。
    pub published_at: &'a str,
}

/// 往哔哩视频数组末尾加一条。相同 BV 号不重复。
pub fn insert_bili(source: &str, draft: BiliDraft<'_>) -> Result<(String, bool), SourceError> {
    let bvid = draft.bvid.trim();
    let title = draft.title.trim();
    if !bvid.starts_with("BV")
        || !bvid.chars().all(|ch| ch.is_ascii_alphanumeric())
        || !(6..=16).contains(&bvid.len())
    {
        return Err(SourceError::Invalid("哔哩视频需要 BV 号"));
    }
    if title.is_empty() {
        return Err(SourceError::Invalid("哔哩视频需要标题"));
    }
    let cover = draft.cover.trim();
    if !cover.is_empty() && !public_http_url(cover) {
        return Err(SourceError::Invalid(
            "封面 URL 必须是公开的 http 或 https 地址",
        ));
    }
    if source.contains(&format!("bvid: {}", js_string(bvid))) {
        return Ok((source.to_string(), false));
    }
    let card = format!(
        "  {{\n    bvid: {},\n    title: {},\n    description: {},\n    cover: {},\n    duration: {},\n    publishedAt: {},\n  }},\n",
        js_string(bvid),
        js_string(title),
        js_string(draft.description.trim()),
        js_string(cover),
        js_string(draft.duration.trim()),
        js_string(draft.published_at.trim())
    );
    Ok((
        insert_card(source, "export const ownerVideos = [", &card)?,
        true,
    ))
}

/// 按 BV 号删除一条哔哩视频。
pub fn remove_bili(source: &str, bvid: &str) -> Result<String, SourceError> {
    remove_record(
        source,
        "export const ownerVideos = [",
        "bvid",
        bvid,
        "没有这条哔哩视频",
    )
}

/// 星空馆项目。封面必须是已经上传到 S3 的公开地址。
pub struct ProjectDraft<'a> {
    /// 稳定 id。
    pub id: &'a str,
    /// 标题。
    pub title: &'a str,
    /// 简介。
    pub summary: &'a str,
    /// 封面。
    pub cover: &'a str,
    /// 项目地址。可以空，空则用站点首页。
    pub url: &'a str,
    /// 标签，逗号分隔。
    pub tags: &'a str,
    /// 核对日期 `YYYY-MM-DD`。
    pub reviewed_at: &'a str,
}

/// 往星空馆项目数组末尾加一个项目。
pub fn insert_project(source: &str, draft: ProjectDraft<'_>) -> Result<String, SourceError> {
    let title = draft.title.trim();
    let summary = draft.summary.trim();
    let cover = draft.cover.trim();
    if !record_id(draft.id) {
        return Err(SourceError::Invalid("星空馆项目无效"));
    }
    if title.is_empty() || summary.is_empty() {
        return Err(SourceError::Invalid("星空馆项目需要标题和简介"));
    }
    if !public_http_url(cover) {
        return Err(SourceError::Invalid(
            "封面 URL 必须是公开的 http 或 https 地址",
        ));
    }
    let url = draft.url.trim();
    let url = if url.is_empty() {
        "https://www.miku.coffee/"
    } else if public_http_url(url) {
        url
    } else {
        return Err(SourceError::Invalid(
            "项目 URL 必须是公开的 http 或 https 地址",
        ));
    };
    let tags = draft
        .tags
        .split([',', '，'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(js_string)
        .collect::<Vec<_>>();
    let tags = if tags.is_empty() {
        js_string("星空馆")
    } else {
        tags.join(", ")
    };
    let card = format!(
        "  {{\n    id: {},\n    title: {},\n    summary: {},\n    author: \"Spume\",\n    category: \"项目\",\n    tags: [{tags}],\n    cover: {},\n    preview: {},\n    url: {},\n    source: {},\n    docs: {},\n    reviewedAt: {},\n    license: \"站长发布\",\n    licenseUrl: {},\n    isGuide: false,\n    demo: {{ kind: \"link\", note: {}, label: \"打开项目\", url: {} }},\n    sections: [[\"项目介绍\", {}, \"\"]],\n  }},\n",
        js_string(draft.id),
        js_string(title),
        js_string(summary),
        js_string(cover),
        js_string(cover),
        js_string(url),
        js_string(url),
        js_string(url),
        js_string(draft.reviewed_at),
        js_string(url),
        js_string(summary),
        js_string(url),
        js_string(summary)
    );
    insert_card(source, "export const projects = [", &card)
}

/// 按 id 删除一个星空馆项目。
pub fn remove_project(source: &str, id: &str) -> Result<String, SourceError> {
    remove_record(
        source,
        "export const projects = [",
        "id",
        id,
        "没有这个星空馆项目",
    )
}

/// 从相册里删一张图。最后一张图被删掉时，整个相册一起去掉。
pub fn remove_gallery_image(
    source: &str,
    album_id: &str,
    image_url: &str,
) -> Result<String, SourceError> {
    if album_id.trim().is_empty() || !public_http_url(image_url.trim()) {
        return Err(SourceError::Invalid("请选择要删除的相册图片"));
    }
    let (start, end) = find_object(
        source,
        "export const galleryAlbums = [",
        "id",
        album_id.trim(),
        "没有这个相册",
    )?;
    let object = source
        .get(start..end)
        .ok_or(SourceError::Invalid("相册对象无效"))?;
    let images_at = object
        .find("images:")
        .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let open = start
        + images_at
        + object[images_at..]
            .find('[')
            .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let close = find_matching(source, open, b'[', b']')?;
    let quoted = js_string(image_url.trim());
    let body = source
        .get(open..=close)
        .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let Some(rel) = body.find(&quoted) else {
        return Err(SourceError::Invalid("没有这张相册图片"));
    };
    let token_at = open + rel;
    let mut next = excise(source, token_at, token_at + quoted.len())?;
    let (start, end) = find_object(
        &next,
        "export const galleryAlbums = [",
        "id",
        album_id.trim(),
        "没有这个相册",
    )?;
    let object = next
        .get(start..end)
        .ok_or(SourceError::Invalid("相册对象无效"))?
        .to_string();
    let images_at = object
        .find("images:")
        .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let open = images_at
        + object[images_at..]
            .find('[')
            .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    let close = find_matching(&object, open, b'[', b']')?;
    let images = object
        .get(open..=close)
        .ok_or(SourceError::Invalid("相册缺少 images 数组"))?;
    if !images.contains('"') {
        return excise(&next, start, end);
    }
    let old_cover = format!("cover: {}", js_string(image_url.trim()));
    if object.contains(&old_cover) {
        let Some(first) = images.split('"').nth(1) else {
            return Err(SourceError::Invalid("相册缺少 images 数组"));
        };
        let replacement = format!("cover: {}", js_string(first));
        let swapped = object.replacen(&old_cover, &replacement, 1);
        next.replace_range(start..end, &swapped);
    }
    Ok(next)
}

/// 按年份、日期、分类和正文删掉一条碎语。
pub fn remove_moment(
    source: &str,
    year: &str,
    date: &str,
    kind: &str,
    content: &str,
) -> Result<String, SourceError> {
    let (open, close) = array_bounds(source, "export const moments = [")?;
    let ranges = object_ranges(source, open, close)?;
    let content = content.trim();
    let found = ranges.into_iter().find(|(start, end)| {
        let Some(object) = source.get(*start..*end) else {
            return false;
        };
        read_field(object, "year").as_deref() == Some(year.trim())
            && read_field(object, "date").as_deref() == Some(date.trim())
            && read_field(object, "type").as_deref() == Some(kind.trim())
            && (content.is_empty()
                || object.contains(&js_string(content))
                || joined_lines(object) == content)
    });
    let Some((start, end)) = found else {
        return Err(SourceError::Invalid("没有这条碎语"));
    };
    excise(source, start, end)
}

fn pretty_journal(root: &Value) -> Result<String, SourceError> {
    let text =
        serde_json::to_string_pretty(root).map_err(|_| SourceError::Invalid("建站日志数据无效"))?;
    Ok(format!("{text}\n"))
}

fn parse_journal(source: &str) -> Result<Value, SourceError> {
    serde_json::from_str(source).map_err(|_| SourceError::Invalid("建站日志数据无效"))
}

fn journal_articles(root: &mut Value) -> Result<&mut Vec<Value>, SourceError> {
    root.get_mut("articles")
        .and_then(Value::as_array_mut)
        .ok_or(SourceError::Invalid("建站日志数据无效"))
}

fn plain_html(body: &str, cover: &str) -> String {
    let mut out = String::new();
    if !cover.is_empty() {
        out.push_str("<p><img src=\"");
        out.push_str(&html_escape(cover));
        out.push_str("\" alt=\"\"></p>");
    }
    for para in body.split("\n\n") {
        let text = para.trim();
        if text.is_empty() {
            continue;
        }
        out.push_str("<p>");
        out.push_str(&html_escape(text).replace('\n', "<br>"));
        out.push_str("</p>");
    }
    out
}

fn yaml_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ")
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn record_id(id: &str) -> bool {
    (4..=32).contains(&id.len()) && id.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn public_http_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit('@').next().unwrap_or("");
    !host.is_empty() && !host.starts_with(':')
}

fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' | '"' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn insert_card(source: &str, marker: &str, card: &str) -> Result<String, SourceError> {
    let (_, close) = array_bounds(source, marker)?;
    let line = source[..close]
        .rfind('\n')
        .ok_or(SourceError::Invalid("数据数组无效"))?
        + 1;
    let head = source
        .get(..line)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    let tail = source
        .get(line..)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    Ok(format!("{head}{card}{tail}"))
}

fn remove_record(
    source: &str,
    marker: &str,
    key: &str,
    value: &str,
    missing: &'static str,
) -> Result<String, SourceError> {
    if value.trim().is_empty() {
        return Err(SourceError::Invalid(missing));
    }
    let (start, end) = find_object(source, marker, key, value.trim(), missing)?;
    excise(source, start, end)
}

fn find_object(
    source: &str,
    marker: &str,
    key: &str,
    value: &str,
    missing: &'static str,
) -> Result<(usize, usize), SourceError> {
    let (open, close) = array_bounds(source, marker)?;
    let needle = format!("{key}: {}", js_string(value));
    object_ranges(source, open, close)?
        .into_iter()
        .find(|(start, end)| {
            source
                .get(*start..*end)
                .is_some_and(|object| object.contains(&needle))
        })
        .ok_or(SourceError::Invalid(missing))
}

fn array_bounds(source: &str, marker: &str) -> Result<(usize, usize), SourceError> {
    let marker_at = source
        .find(marker)
        .ok_or(SourceError::Invalid("未找到数据导出"))?;
    let open = marker_at
        + source[marker_at..]
            .find('[')
            .ok_or(SourceError::Invalid("未找到数据导出"))?;
    let close = find_matching(source, open, b'[', b']')?;
    Ok((open, close))
}

fn object_ranges(
    source: &str,
    open: usize,
    close: usize,
) -> Result<Vec<(usize, usize)>, SourceError> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut index = open + 1;
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    let mut object_start = None;
    while index < close {
        let Some(&ch) = bytes.get(index) else {
            break;
        };
        if let Some(mark) = quote {
            if escaped {
                escaped = false;
            } else if ch == b'\\' {
                escaped = true;
            } else if ch == mark {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(ch, b'"' | b'\'' | b'`') {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == b'{' {
            if depth == 0 {
                object_start = Some(index);
            }
            depth += 1;
        } else if ch == b'}' {
            depth -= 1;
            if depth == 0 {
                let start = object_start
                    .take()
                    .ok_or(SourceError::Invalid("数据对象无效"))?;
                ranges.push((start, index + 1));
            }
        }
        index += 1;
    }
    Ok(ranges)
}

fn find_matching(
    source: &str,
    open_index: usize,
    open: u8,
    close: u8,
) -> Result<usize, SourceError> {
    let bytes = source.as_bytes();
    if bytes.get(open_index).copied() != Some(open) {
        return Err(SourceError::Invalid("括号位置无效"));
    }
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (index, &ch) in bytes.iter().enumerate().skip(open_index) {
        if let Some(mark) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == b'\\' {
                escaped = true;
                continue;
            }
            if ch == mark {
                quote = None;
            }
            continue;
        }
        if matches!(ch, b'"' | b'\'' | b'`') {
            quote = Some(ch);
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        }
    }
    Err(SourceError::Invalid("括号没有闭合"))
}

fn excise(source: &str, token_start: usize, token_end: usize) -> Result<String, SourceError> {
    let mut start = token_start;
    if let Some(nl) = source[..start].rfind('\n') {
        let between = source.get(nl + 1..start).unwrap_or("");
        if between.chars().all(|ch| ch == ' ' || ch == '\t') {
            start = nl;
        }
    }
    let bytes = source.as_bytes();
    let mut end = token_end;
    while matches!(bytes.get(end), Some(b' ' | b'\t')) {
        end += 1;
    }
    if bytes.get(end) == Some(&b',') {
        end += 1;
    }
    while matches!(bytes.get(end), Some(b' ' | b'\t' | b'\r')) {
        end += 1;
    }
    if bytes.get(end) == Some(&b'\n') {
        end += 1;
    }
    let head = source
        .get(..start)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    let tail = source
        .get(end..)
        .ok_or(SourceError::Invalid("数据数组无效"))?;
    Ok(format!("{head}{tail}"))
}

fn read_field(object: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}: \"");
    let at = object.find(&pattern)?;
    let rest = object.get(at + pattern.len()..)?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            out.push(chars.next()?);
            continue;
        }
        if ch == '"' {
            return Some(out);
        }
        out.push(ch);
    }
    None
}

fn joined_lines(object: &str) -> String {
    let Some(at) = object.find("lines:") else {
        return String::new();
    };
    let Some(open) = object[at..].find('[') else {
        return String::new();
    };
    let Ok(close) = find_matching(object, at + open, b'[', b']') else {
        return String::new();
    };
    let Some(body) = object.get(at + open..=close) else {
        return String::new();
    };
    let mut lines = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        let mut end = 0;
        let mut found = false;
        let mut escaped = false;
        for (index, ch) in after.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                end = index;
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
        lines.push(after[..end].replace("\\n", "\n").replace("\\\"", "\""));
        let Some(next) = after.get(end + 1..) else {
            break;
        };
        rest = next;
    }
    lines.join("\n")
}
