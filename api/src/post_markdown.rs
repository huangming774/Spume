//! 把站长控制台的文章草稿收成 Hexo 文章。
//!
//! 路径和 front matter 跟原来的 Go 发布保持一致：日期保留草稿里已有的值，
//! 封面参数优先于草稿里的封面。

use chrono::{DateTime, Utc};
use thiserror::Error;

const TITLE_MAX: usize = 120;
const BODY_MAX: usize = 200_000;
const PATH_PREFIX: &str = "blog/source/_posts/";

/// 一篇待发布的文章。字段借用请求体。
pub struct ArticleInput<'a> {
    /// 标题。
    pub title: &'a str,
    /// Markdown 正文，可以自带 front matter。
    pub body: &'a str,
    /// 封面地址。空字符串表示沿用正文里的封面。
    pub cover_url: &'a str,
}

/// 写进仓库的文章文件。
pub struct ArticleFile {
    /// `blog/source/_posts/` 下的路径。
    pub path: String,
    /// 含 front matter 的 Markdown。
    pub markdown: String,
}

/// 草稿不能发布。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArticleError {
    /// 标题为空或超长，或正文为空、过大。
    #[error("{0}")]
    Invalid(&'static str),
}

/// 生成文章路径和正文。`now` 用在文件名和没有日期的 front matter 上，按 UTC。
pub fn build_article(
    input: ArticleInput<'_>,
    now: DateTime<Utc>,
) -> Result<ArticleFile, ArticleError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(ArticleError::Invalid("发布标题不能为空。"));
    }
    if title.chars().count() > TITLE_MAX {
        return Err(ArticleError::Invalid("发布标题太长。"));
    }
    if input.body.trim().is_empty() {
        return Err(ArticleError::Invalid("发布正文不能为空。"));
    }
    if input.body.len() > BODY_MAX {
        return Err(ArticleError::Invalid("发布正文太大。"));
    }
    let normalized = input
        .body
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n");
    let (front, body) = split_front_matter(&normalized);
    let markdown = merge_front_matter(front, title, input.cover_url.trim(), now, body);
    Ok(ArticleFile {
        path: format!("{PATH_PREFIX}{}.md", filename_stem(title, now)),
        markdown,
    })
}

/// 文件已存在时换一个带时间的路径，避免覆盖旧文章。
pub fn conflict_path(path: &str, now: DateTime<Utc>) -> String {
    let suffix = now.format("-%Y%m%d-%H%M%S");
    match path.strip_suffix(".md") {
        Some(stem) => format!("{stem}{suffix}.md"),
        None => format!("{path}{suffix}"),
    }
}

fn split_front_matter(content: &str) -> (&str, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return ("", content);
    };
    if let Some(end) = rest.find("\n---\n") {
        return (&rest[..end], &rest[end + 5..]);
    }
    if let Some(front) = rest.strip_suffix("\n---") {
        return (front, "");
    }
    ("", content)
}

fn merge_front_matter(
    front: &str,
    title: &str,
    cover_url: &str,
    now: DateTime<Utc>,
    body: &str,
) -> String {
    let mut date = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut cover = cover_url.to_string();
    let mut others = Vec::new();
    for line in front.trim_matches('\n').lines() {
        let Some((key, value)) = front_line(line) else {
            others.push(line.to_string());
            continue;
        };
        match key {
            "title" => {}
            "date" => {
                let kept = unquote(value);
                if !kept.is_empty() {
                    date = kept;
                }
            }
            "cover" if cover.is_empty() => cover = unquote(value),
            "cover" => {}
            _ => others.push(line.to_string()),
        }
    }
    let mut lines = vec![
        format!("title: {}", yaml_scalar(title)),
        format!("date: {date}"),
    ];
    if !cover.is_empty() {
        lines.push(format!("cover: {}", yaml_scalar(&cover)));
    }
    lines.extend(others);
    let content = body.trim_start_matches('\n');
    let content = if content.trim().is_empty() {
        format!("# {title}\n")
    } else {
        format!("{}\n", content.trim_end_matches('\n'))
    };
    format!("---\n{}\n---\n\n{content}", lines.join("\n"))
}

fn front_line(line: &str) -> Option<(&str, &str)> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (key, value) = line.split_once(':')?;
    if key.is_empty() {
        return None;
    }
    Some((key.trim(), value.trim()))
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn yaml_scalar(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let special = value
        .chars()
        .any(|ch| matches!(ch, '#' | '"' | '\'' | '[' | ']' | '{' | '}'));
    let edges = value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace);
    if special || edges {
        quote(value)
    } else {
        value.to_string()
    }
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' | '"' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn filename_stem(title: &str, now: DateTime<Utc>) -> String {
    let mut stem = String::new();
    let mut last_dash = false;
    for ch in title.trim().chars() {
        if ch.is_alphanumeric() {
            stem.push(if ch.is_ascii() {
                ch.to_ascii_lowercase()
            } else {
                ch
            });
            last_dash = false;
        } else if !last_dash && !stem.is_empty() {
            stem.push('-');
            last_dash = true;
        }
    }
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        format!("post-{}", now.format("%Y%m%d-%H%M%S"))
    } else {
        stem.to_string()
    }
}
