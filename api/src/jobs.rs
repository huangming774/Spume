//! QStash 调用的维护、索引和生图补存。
//!
//! 读 Markdown 用的是普通文件读取。一次索引只有几十篇短文，不单独开阻塞线程池。

use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use thiserror::Error;

use crate::image_store::{self, ImageStoreError};
use crate::AppState;

const CHUNK_CHARS: usize = 700;

/// 一条要写入向量索引的文档。
pub struct ContentDoc {
    /// `blog:slug:0` 这种稳定 id。
    pub id: String,
    /// 交给 Upstash 内置模型的文本。
    pub data: String,
    /// 搜索结果标题。
    pub title: String,
    /// 站内路径。
    pub url: String,
    /// 摘要。
    pub excerpt: String,
    /// `blog` 或 `journal`。
    pub kind: String,
}

/// 维护或索引失败。
#[derive(Debug, Error)]
pub enum JobError {
    /// 没有数据库。
    #[error("数据库还没有连上")]
    NoDatabase,
    /// 数据库语句失败。
    #[error("{0}")]
    Database(String),
    /// 内容目录或文件读不出来。
    #[error("{0}")]
    Content(String),
    /// 向量库没配或拒绝了请求。
    #[error("{0}")]
    Vector(String),
    /// 图片记录不存在。
    #[error("图片不存在")]
    NotFound,
    /// 转存失败。
    #[error("{0}")]
    Image(String),
}

/// 清理过期会话、过期安全问答，以及 120 天前的聊天小时统计。
pub async fn maintain(state: &AppState) -> Result<Value, JobError> {
    let pool = state.db.as_ref().ok_or(JobError::NoDatabase)?;
    let sessions = delete_count(pool, "DELETE FROM sessions WHERE expires_at < now()").await?;
    let challenges = delete_count(
        pool,
        "DELETE FROM login_challenges WHERE expires_at < now()",
    )
    .await?;
    let cutoff = (Utc::now() - Duration::days(120))
        .format("%Y-%m-%dT%H")
        .to_string();
    let hourly = sqlx::query("DELETE FROM ai_chat_hourly WHERE bucket < $1")
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(|error| JobError::Database(error.to_string()))?
        .rows_affected();
    Ok(json!({"sessions": sessions, "challenges": challenges, "hourly": hourly}))
}

/// 把文章和建站日志写入 Upstash Vector。先按前缀删掉旧条目，再整批写入。
pub async fn index_content(state: &AppState) -> Result<usize, JobError> {
    if !state.cfg.vector_ready() {
        return Err(JobError::Vector("向量库还没有配置".into()));
    }
    let root = content_root(&state.cfg.content_root)?;
    let mut docs = Vec::new();
    collect_dir(&root.join("blog/source/_posts"), "blog", "/blog", &mut docs)?;
    collect_dir(
        &root.join("build/content/articles"),
        "journal",
        "/build/post",
        &mut docs,
    )?;
    delete_prefix(state, "blog:").await?;
    delete_prefix(state, "journal:").await?;
    if docs.is_empty() {
        return Ok(0);
    }
    upsert(state, &docs).await?;
    Ok(docs.len())
}

/// 如果这张生成图还停在临时地址，就再转存一次到 R2。已经有对象键则直接返回。
pub async fn restock_image(state: &AppState, id: i64) -> Result<Value, JobError> {
    let pool = state.db.as_ref().ok_or(JobError::NoDatabase)?;
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT image_url, object_key FROM ai_images WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|error| JobError::Database(error.to_string()))?;
    let Some((url, key)) = row else {
        return Err(JobError::NotFound);
    };
    if !key.is_empty() {
        return Ok(json!({"id": id, "stored": true, "url": url, "path": key}));
    }
    let stored = image_store::store_remote_image(state, &url)
        .await
        .map_err(store_error)?;
    sqlx::query("UPDATE ai_images SET image_url = $1, object_key = $2 WHERE id = $3")
        .bind(&stored.url)
        .bind(&stored.key)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|error| JobError::Database(error.to_string()))?;
    Ok(json!({"id": id, "stored": true, "url": stored.url, "path": stored.key}))
}

/// 探测一个已配置服务。没配返回 `missing`，请求失败或状态不是成功返回 `down`。
pub async fn probe(
    state: &AppState,
    url: &str,
    token: &str,
    client_error_is_up: bool,
) -> &'static str {
    if url.is_empty() {
        return "missing";
    }
    let mut request = state
        .http
        .get(url)
        .timeout(std::time::Duration::from_secs(3));
    if !token.is_empty() {
        request = request.bearer_auth(token);
    }
    match request.send().await {
        Ok(response) if response.status().is_success() => "up",
        Ok(response) if client_error_is_up && response.status().is_client_error() => "up",
        _ => "down",
    }
}

/// 从一篇 Markdown 切出索引文档。
pub fn documents(kind: &str, slug: &str, url: &str, markdown: &str) -> Vec<ContentDoc> {
    let (title, body) = title_and_body(markdown, slug);
    let excerpt = excerpt_of(&body);
    let pieces = chunks(&body);
    let pieces = if pieces.is_empty() {
        vec![title.clone()]
    } else {
        pieces
    };
    pieces
        .into_iter()
        .enumerate()
        .map(|(index, piece)| ContentDoc {
            id: format!("{kind}:{slug}:{index}"),
            data: format!("{title}\n\n{piece}"),
            title: title.clone(),
            url: url.to_string(),
            excerpt: excerpt.clone(),
            kind: kind.to_string(),
        })
        .collect()
}

fn store_error(error: ImageStoreError) -> JobError {
    JobError::Image(error.to_string())
}

async fn delete_count(pool: &sqlx::PgPool, sql: &str) -> Result<u64, JobError> {
    Ok(sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|error| JobError::Database(error.to_string()))?
        .rows_affected())
}

fn content_root(configured: &str) -> Result<PathBuf, JobError> {
    if !configured.trim().is_empty() {
        let path = PathBuf::from(configured.trim());
        return if path.join("blog/source/_posts").is_dir() {
            Ok(path)
        } else {
            Err(JobError::Content(format!(
                "{} 下面没有 blog/source/_posts",
                path.display()
            )))
        };
    }
    for candidate in [PathBuf::from("."), PathBuf::from("..")] {
        if candidate.join("blog/source/_posts").is_dir() {
            return Ok(candidate);
        }
    }
    Err(JobError::Content(
        "没有找到文章目录。设置 ACG_CONTENT_ROOT 指向仓库根目录。".into(),
    ))
}

fn collect_dir(
    dir: &Path,
    kind: &str,
    url_prefix: &str,
    docs: &mut Vec<ContentDoc>,
) -> Result<(), JobError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|error| JobError::Content(format!("{}: {error}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|error| JobError::Content(error.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .trim();
        if slug.is_empty() {
            return Err(JobError::Content(format!("{} 没有文件名", path.display())));
        }
        let markdown = std::fs::read_to_string(&path)
            .map_err(|error| JobError::Content(format!("{}: {error}", path.display())))?;
        docs.extend(documents(
            kind,
            slug,
            &format!("{url_prefix}/{slug}"),
            &markdown,
        ));
    }
    Ok(())
}

async fn delete_prefix(state: &AppState, prefix: &str) -> Result<(), JobError> {
    vector_post(state, "delete", &json!({"prefix": prefix}))
        .await
        .map(|_| ())
}

async fn upsert(state: &AppState, docs: &[ContentDoc]) -> Result<(), JobError> {
    let payload: Vec<Value> = docs
        .iter()
        .map(|doc| json!({"id": doc.id, "data": doc.data, "metadata": {"title": doc.title, "url": doc.url, "excerpt": doc.excerpt, "type": doc.kind}}))
        .collect();
    vector_post(state, "upsert", &json!(payload))
        .await
        .map(|_| ())
}

async fn vector_post(state: &AppState, path: &str, body: &Value) -> Result<Value, JobError> {
    let url = format!("{}/{}", state.cfg.vector_url.trim_end_matches('/'), path);
    let response = state
        .http
        .post(url)
        .bearer_auth(&state.cfg.vector_token)
        .json(body)
        .send()
        .await
        .map_err(|error| JobError::Vector(error.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| JobError::Vector(error.to_string()))?;
    if !status.is_success() {
        return Err(JobError::Vector(format!(
            "向量库返回 HTTP {}: {}",
            status.as_u16(),
            trim_text(&text)
        )));
    }
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&text).map_err(|error| JobError::Vector(error.to_string()))
}

fn title_and_body(markdown: &str, slug: &str) -> (String, String) {
    let normalized = markdown
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n");
    let (front, body) = split_front(&normalized);
    let title = front_value(front, "title")
        .filter(|value| !value.is_empty())
        .or_else(|| heading(body))
        .unwrap_or_else(|| slug.to_string());
    (title, body.trim().to_string())
}

fn split_front(content: &str) -> (&str, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return ("", content);
    };
    if let Some(end) = rest.find("\n---\n") {
        return (&rest[..end], &rest[end + 5..]);
    }
    ("", content)
}

fn front_value(front: &str, key: &str) -> Option<String> {
    for line in front.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim() == key {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn heading(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        line.strip_prefix("# ")
            .map(|title| title.trim().to_string())
            .filter(|title| !title.is_empty())
    })
}

fn excerpt_of(body: &str) -> String {
    body.chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect::<String>()
        .trim()
        .to_string()
}

fn chunks(body: &str) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + CHUNK_CHARS).min(chars.len());
        let piece: String = chars[start..end].iter().collect();
        let piece = piece.trim().to_string();
        if !piece.is_empty() {
            out.push(piece);
        }
        if end == chars.len() {
            break;
        }
        start = end;
    }
    out
}

fn trim_text(value: &str) -> String {
    value.chars().take(180).collect()
}
