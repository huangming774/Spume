//! 按 IP 限流。没配 Redis 时用进程内滑动窗口；配了 Redis 时用一分钟固定窗口。
//!
//! 锁只包住内存表，不跨过 await。锁中毒时取回内部数据继续用，避免把一次
//! 恐慌变成后续所有请求都失败。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 一个限流窗口。可以克隆，多个请求共享同一张表。
#[derive(Clone)]
pub struct RateGate {
    inner: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    limit: usize,
    window: Duration,
}

impl RateGate {
    /// `limit` 是窗口内允许的次数。
    pub fn new(limit: usize, window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window,
        }
    }

    /// 记录一次访问。窗口已满时返回 false，并且不写入这次访问。
    pub fn allow(&self, key: &str) -> bool {
        let mut table = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();
        let hits = table.entry(key.to_string()).or_default();
        hits.retain(|at| now.saturating_duration_since(*at) < self.window);
        if hits.len() >= self.limit {
            return false;
        }
        hits.push(now);
        true
    }
}

/// 从 Upstash Redis 的 pipeline 响应里取出 `INCR` 的计数。
pub fn pipeline_count(body: &serde_json::Value) -> Result<i64, String> {
    let count = body
        .get(0)
        .and_then(|item| item.get("result"))
        .or_else(|| body.get("result"))
        .and_then(serde_json::Value::as_i64);
    count.ok_or_else(|| format!("限流结果无法解析: {body}"))
}

/// 用 Redis 的 `INCR` 加 `EXPIRE` 判断这一分钟是否还允许访问。
pub async fn allow_fixed_window(
    http: &reqwest::Client,
    redis_url: &str,
    token: &str,
    key: &str,
    limit: i64,
) -> Result<bool, String> {
    let response = http
        .post(redis_url.trim_end_matches('/'))
        .bearer_auth(token)
        .json(&serde_json::json!([["INCR", key], ["EXPIRE", key, "70"]]))
        .send()
        .await
        .map_err(|error| format!("Redis 没有回应: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Redis 返回 HTTP {}", response.status()));
    }
    let body = response
        .json::<serde_json::Value>()
        .await
        .map_err(|error| format!("Redis 结果无法解析: {error}"))?;
    Ok(pipeline_count(&body)? <= limit)
}
