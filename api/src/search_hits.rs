//! 把 Upstash Vector 的查询结果收成主站搜索要用的四字段。

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// 一条可点击的搜索结果。字符串从上游 JSON 拷出，因为原 `Value` 不会继续存活。
#[derive(Debug, PartialEq, Serialize)]
pub struct SearchHit {
    /// 文章或日志标题。
    pub title: String,
    /// 站内路径或绝对地址。
    pub url: String,
    /// 摘要。上游没给时为空字符串。
    pub excerpt: String,
    /// 向量相似度。
    pub score: f64,
}

/// 上游 JSON 不是 Vector 查询结果。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SearchHitError {
    /// 缺少 `result` 数组。
    #[error("检索结果缺少 result 数组")]
    MissingResult,
    /// 某一条没有可用的标题和地址。
    #[error("检索结果第 {index} 条缺少标题或地址")]
    Incomplete { index: usize },
    /// `score` 存在但不是数字。
    #[error("检索结果第 {index} 条的相似度不是数字")]
    BadScore { index: usize },
}

/// 读取 Vector `/query` 的 JSON。`result` 必须是数组，每条都要有标题和地址。
pub fn hits_from_vector(body: &Value) -> Result<Vec<SearchHit>, SearchHitError> {
    let rows = body
        .get("result")
        .and_then(Value::as_array)
        .ok_or(SearchHitError::MissingResult)?;
    let mut hits = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        hits.push(one_hit(index, row)?);
    }
    Ok(hits)
}

fn one_hit(index: usize, row: &Value) -> Result<SearchHit, SearchHitError> {
    let metadata = row.get("metadata");
    let title = text_field(row, metadata, "title");
    let url = text_field(row, metadata, "url");
    if title.is_empty() || url.is_empty() {
        return Err(SearchHitError::Incomplete { index });
    }
    let excerpt = text_field(row, metadata, "excerpt");
    let score = match row.get("score") {
        None => 0.0,
        Some(value) => value.as_f64().ok_or(SearchHitError::BadScore { index })?,
    };
    Ok(SearchHit {
        title,
        url,
        excerpt,
        score,
    })
}

fn text_field(row: &Value, metadata: Option<&Value>, key: &str) -> String {
    metadata
        .and_then(|item| item.get(key))
        .or_else(|| row.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}
