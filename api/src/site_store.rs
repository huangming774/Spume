//! 站长发布的正文。图片文件在 S3，这里只保存正文和图片地址。

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use thiserror::Error;

const KINDS: [&str; 7] = [
    "about",
    "gallery",
    "moment",
    "bili",
    "article",
    "journal",
    "project",
];

/// 一条待写入的内容。
pub struct NewEntry {
    /// `about`、`gallery`、`moment`、`bili`、`article`、`journal`、`project`。
    pub kind: String,
    /// 标题。相册用相册名，碎语用分类。
    pub title: String,
    /// 正文。相册可以空。
    pub body: String,
    /// S3 公开地址。没有图片时为空。
    pub image_url: String,
    /// 栏目自己的字段，例如 BV 号、相册 id。
    pub extra: Value,
}

/// 已经落库的一条内容。
#[derive(Clone)]
pub struct Entry {
    /// 数据库 id。
    pub id: i64,
    /// 栏目。
    pub kind: String,
    /// 标题。
    pub title: String,
    /// 正文。
    pub body: String,
    /// 图片地址。
    pub image_url: String,
    /// 附加字段。
    pub extra: Value,
    /// 写入时间。
    pub created_at: DateTime<Utc>,
}

/// 内容没有写上。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    /// 草稿本身不能保存。
    #[error("{0}")]
    Invalid(&'static str),
    /// 数据库拒绝了写入或查询。
    #[error("{0}")]
    Database(String),
}

/// 这种栏目是否允许公开读取。
pub fn known_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// 检查草稿。通过后的字符串已经去掉首尾空白。
pub fn prepare(entry: NewEntry) -> Result<NewEntry, StoreError> {
    checked(entry)
}
pub async fn insert(pool: &PgPool, entry: NewEntry) -> Result<Entry, StoreError> {
    let entry = checked(entry)?;
    let extra = serde_json::to_string(&entry.extra).unwrap_or_else(|_| "{}".to_string());
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>)>(
        "INSERT INTO site_entries (kind, title, body, image_url, extra) VALUES ($1, $2, $3, $4, $5::jsonb) RETURNING id, created_at",
    )
    .bind(&entry.kind)
    .bind(&entry.title)
    .bind(&entry.body)
    .bind(&entry.image_url)
    .bind(extra)
    .fetch_one(pool)
    .await
    .map_err(|error| StoreError::Database(error.to_string()))?;
    Ok(Entry {
        id: row.0,
        kind: entry.kind,
        title: entry.title,
        body: entry.body,
        image_url: entry.image_url,
        extra: entry.extra,
        created_at: row.1,
    })
}

/// 按栏目列出内容，新的在前。
pub async fn list(pool: &PgPool, kind: &str) -> Result<Vec<Entry>, StoreError> {
    if !known_kind(kind) {
        return Err(StoreError::Invalid("栏目无效"));
    }
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>)>(
        "SELECT id, kind, title, body, image_url, extra::text, created_at FROM site_entries WHERE kind = $1 ORDER BY created_at DESC, id DESC",
    )
    .bind(kind)
    .fetch_all(pool)
    .await
    .map_err(|error| StoreError::Database(error.to_string()))?;
    rows_to_entries(rows)
}

/// 跨栏目列出最近发布的内容。
pub async fn list_recent(pool: &PgPool, limit: i64) -> Result<Vec<Entry>, StoreError> {
    let limit = limit.clamp(1, 40);
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>)>(
        "SELECT id, kind, title, body, image_url, extra::text, created_at FROM site_entries ORDER BY created_at DESC, id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|error| StoreError::Database(error.to_string()))?;
    rows_to_entries(rows)
}

fn rows_to_entries(
    rows: Vec<(i64, String, String, String, String, String, DateTime<Utc>)>,
) -> Result<Vec<Entry>, StoreError> {
    rows.into_iter()
        .map(|(id, kind, title, body, image_url, extra, created_at)| {
            Ok(Entry {
                id,
                kind,
                title,
                body,
                image_url,
                extra: serde_json::from_str(&extra).unwrap_or_else(|_| json!({})),
                created_at,
            })
        })
        .collect()
}

/// 按 id 删除一条内容。栏目不匹配时不删。
pub async fn delete(pool: &PgPool, id: i64, kind: &str) -> Result<bool, StoreError> {
    if id <= 0 || !known_kind(kind) {
        return Err(StoreError::Invalid("要删除的内容无效"));
    }
    let result = sqlx::query("DELETE FROM site_entries WHERE id = $1 AND kind = $2")
        .bind(id)
        .bind(kind)
        .execute(pool)
        .await
        .map_err(|error| StoreError::Database(error.to_string()))?;
    Ok(result.rows_affected() > 0)
}

/// 给接口用的 JSON。
pub fn entry_json(entry: &Entry) -> Value {
    json!({
        "id": entry.id,
        "kind": entry.kind,
        "title": entry.title,
        "body": entry.body,
        "imageUrl": entry.image_url,
        "extra": entry.extra,
        "createdAt": entry.created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
    })
}

fn checked(entry: NewEntry) -> Result<NewEntry, StoreError> {
    let kind = entry.kind.trim();
    if !known_kind(kind) {
        return Err(StoreError::Invalid("栏目无效"));
    }
    let title = entry.title.trim();
    let body = entry.body.trim();
    let image_url = entry.image_url.trim();
    if title.chars().count() > 120 {
        return Err(StoreError::Invalid("标题不能超过 120 个字"));
    }
    if body.len() > 200_000 {
        return Err(StoreError::Invalid("正文过长"));
    }
    if !image_url.is_empty() && !public_http_url(image_url) {
        return Err(StoreError::Invalid("图片地址必须是公开的 http 或 https 地址"));
    }
    let needs_title = !matches!(kind, "gallery");
    if needs_title && title.is_empty() {
        return Err(StoreError::Invalid("标题不能为空"));
    }
    let needs_body = matches!(kind, "about" | "moment" | "article" | "journal" | "project");
    if needs_body && body.is_empty() {
        return Err(StoreError::Invalid("正文不能为空"));
    }
    if matches!(kind, "gallery" | "project") && image_url.is_empty() {
        return Err(StoreError::Invalid("请先把图片上传到 S3"));
    }
    if kind == "bili" {
        let bvid = entry.extra.get("bvid").and_then(Value::as_str).unwrap_or("");
        if !bvid.starts_with("BV")
            || !(6..=16).contains(&bvid.len())
            || !bvid.chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            return Err(StoreError::Invalid("哔哩视频需要 BV 号"));
        }
    }
    Ok(NewEntry {
        kind: kind.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        image_url: image_url.to_string(),
        extra: if entry.extra.is_object() {
            entry.extra
        } else {
            json!({})
        },
    })
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
