//! 把访客问题和当前页面整理成 DeepSeek 的一条用户消息。
//!
//! 页面字段有长度上限。没有页面信息时，用户消息保持原问题。

use serde_json::{json, Value};

/// 系统提示。要求模型把给出的页面内容当作正在看的页面。
pub const SYSTEM_PROMPT: &str = "You are the Taozhiyy site AI assistant. Be concise, friendly, and lightly anime-styled without overdoing it. Answer in the user's language. If current page context is provided, treat it as ground truth for what the user is viewing. Do not invent page content that is not present in the supplied context. Prefer answers about the site, posts, technical content, learning notes, and the current page.";

/// 前端随问题一起送来的页面字段。
pub struct PageFields<'a> {
    /// 页面地址。空字符串时用外层 `pageUrl`。
    pub page_url: &'a str,
    /// 页面标题。空字符串时用外层 `pageTitle`。
    pub page_title: &'a str,
    /// 站内路径。
    pub page_path: &'a str,
    /// 页面语言。
    pub language: &'a str,
    /// 栏目，例如 `guestbook`。
    pub site_section: &'a str,
    /// 标题列表，最多保留 10 条。
    pub headings: &'a [&'a str],
    /// 可见正文。
    pub visible_text: &'a str,
}

/// 拼出发给模型的用户消息。
pub fn user_content(
    message: &str,
    page_url: &str,
    page_title: &str,
    page: PageFields<'_>,
) -> String {
    let message = message.trim();
    let page_url = page_url.trim();
    let page_title = page_title.trim();
    let url = limit_bytes(first_text(page.page_url, page_url).trim(), 400);
    let title = limit_runes(first_text(page.page_title, page_title).trim(), 120);
    let path = limit_runes(page.page_path.trim(), 160);
    let language = limit_runes(page.language.trim(), 30);
    let section = limit_runes(page.site_section.trim(), 60);
    let visible = limit_runes(&collapse_space(page.visible_text), 2200);
    let mut headings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for heading in page.headings {
        let text = limit_runes(&collapse_space(heading), 120);
        if text.is_empty() || !seen.insert(text.clone()) {
            continue;
        }
        headings.push(text);
        if headings.len() == 10 {
            break;
        }
    }
    if page_url.is_empty()
        && page_title.is_empty()
        && url.is_empty()
        && title.is_empty()
        && path.is_empty()
        && language.is_empty()
        && section.is_empty()
        && headings.is_empty()
        && visible.is_empty()
    {
        return message.to_string();
    }
    let mut content = String::from("[Current page context]\n");
    push_line(&mut content, "Title", &title);
    push_line(&mut content, "URL", &url);
    push_line(&mut content, "Path", &path);
    push_line(&mut content, "Section", &section);
    push_line(&mut content, "Language", &language);
    if !headings.is_empty() {
        content.push_str("Headings: ");
        content.push_str(&headings.join(" | "));
        content.push('\n');
    }
    push_line(&mut content, "Visible text", &visible);
    content.push_str("\n[User question]\n");
    content.push_str(message);
    content
}

/// DeepSeek 请求体。温度 0.7，最多 400 个 token。
pub fn request_body(model: &str, content: &str) -> Value {
    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": content}
        ],
        "temperature": 0.7,
        "max_tokens": 400
    })
}

fn push_line(content: &mut String, label: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    content.push_str(label);
    content.push_str(": ");
    content.push_str(value);
    content.push('\n');
}

fn first_text<'a>(primary: &'a str, fallback: &'a str) -> &'a str {
    if primary.trim().is_empty() {
        fallback
    } else {
        primary
    }
}

fn collapse_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn limit_runes(value: &str, max_len: usize) -> String {
    let count = value.chars().count();
    if count <= max_len {
        return value.to_string();
    }
    let mut clipped: String = value.chars().take(max_len).collect();
    clipped.push_str("...");
    clipped
}

fn limit_bytes(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    let mut end = max_len;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &value[..end])
}
