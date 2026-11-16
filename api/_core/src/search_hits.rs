//! 把 Zilliz Cloud（Milvus REST v2）BM25 检索的响应收成主站搜索要用的四字段。

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
    /// BM25 相关度（Milvus 返回的 distance，越大越相关）。
    pub score: f64,
}

/// 上游 JSON 不是 Vector 查询结果。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SearchHitError {
    /// 缺少 `data` 命中数组。
    #[error("检索结果缺少 data 数组")]
    MissingResult,
    /// 某一条没有可用的标题和地址。
    #[error("检索结果第 {index} 条缺少标题或地址")]
    Incomplete { index: usize },
    /// `distance` 存在但不是数字。
    #[error("检索结果第 {index} 条的相似度不是数字")]
    BadScore { index: usize },
}

/// 读取 Milvus `/v2/vectordb/entities/search` 的 JSON。
/// Zilliz Cloud 返回扁平的命中数组；Milvus 自建版是「每个查询一个内层数组」，
/// 两种形状都收。`data` 存在但为空是「没有命中」，不是错误。
pub fn hits_from_milvus(body: &Value) -> Result<Vec<SearchHit>, SearchHitError> {
    let rows = body
        .get("data")
        .and_then(Value::as_array)
        .ok_or(SearchHitError::MissingResult)?;
    let flat: Vec<&Value> = match rows.first() {
        Some(Value::Array(inner)) => inner.iter().collect(),
        Some(Value::Object(_)) => rows.iter().collect(),
        _ => Vec::new(),
    };
    let mut hits = Vec::with_capacity(flat.len());
    for (index, row) in flat.iter().enumerate() {
        hits.push(one_hit(index, row)?);
    }
    Ok(hits)
}

fn one_hit(index: usize, row: &Value) -> Result<SearchHit, SearchHitError> {
    let title = text_field(row, "title");
    let url = text_field(row, "url");
    if title.is_empty() || url.is_empty() {
        return Err(SearchHitError::Incomplete { index });
    }
    let excerpt = text_field(row, "excerpt");
    let score = match row.get("distance").or_else(|| row.get("score")) {
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

fn text_field(row: &Value, key: &str) -> String {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}
