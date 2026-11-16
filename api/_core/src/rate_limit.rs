//! 按 IP 限流。没配 Redis 时用进程内滑动窗口；配了 Redis 时用 ZSET 滑动窗口，
//! 语义和 Go 的 `ipRateLimiter` 一致：被拒的那次访问不计数。
//!
//! 锁只包住内存表，不跨过 await。锁中毒时取回内部数据继续用，避免把一次
//! 恐慌变成后续所有请求都失败。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 滑动窗口长度，一分钟，和 Go 版限流窗口一致。
const WINDOW_MS: i64 = 60_000;

/// 同一毫秒内的多次访问靠这个序号区分，避免 ZADD 撞成员。
static MEMBER_SEQ: AtomicU64 = AtomicU64::new(0);

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

/// 从 Upstash Redis 的 pipeline 响应里取出第 `index` 条命令的计数。
pub fn pipeline_count_at(body: &serde_json::Value, index: usize) -> Result<i64, String> {
    let count = body
        .get(index)
        .and_then(|item| item.get("result"))
        .or_else(|| if index == 0 { body.get("result") } else { None })
        .and_then(serde_json::Value::as_i64);
    count.ok_or_else(|| format!("限流结果无法解析: {body}"))
}

/// 读 pipeline 第一条命令的计数（单条命令的响应是 `{"result": ...}`）。
pub fn pipeline_count(body: &serde_json::Value) -> Result<i64, String> {
    pipeline_count_at(body, 0)
}

/// Upstash 的批量命令要发到 `/pipeline` 端点；发到根地址会被当成单条命令而报 400。
fn pipeline_endpoint(redis_url: &str) -> String {
    format!("{}/pipeline", redis_url.trim_end_matches('/'))
}

/// 用 Redis 的 ZSET 做一分钟滑动窗口，等价于 Go 的 `ipRateLimiter.Allow`：
/// 先删窗口外的记录，把这次访问记进去，再数窗口内的次数。超过 limit 就把
/// 这次记录撤掉（被拒的访问不计数）并返回 false。
pub async fn allow_sliding_window(
    http: &reqwest::Client,
    redis_url: &str,
    token: &str,
    key: &str,
    limit: i64,
) -> Result<bool, String> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("系统时钟不对: {error}"))?
        .as_millis() as i64;
    let member = format!(
        "{now_ms}:{}:{}",
        std::process::id(),
        MEMBER_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let cutoff = now_ms - WINDOW_MS;
    let response = http
        .post(pipeline_endpoint(redis_url))
        .bearer_auth(token)
        .json(&serde_json::json!([
            ["ZREMRANGEBYSCORE", key, "0", cutoff.to_string()],
            ["ZADD", key, now_ms.to_string(), member.as_str()],
            ["ZCARD", key],
            ["EXPIRE", key, "70"],
        ]))
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
    if pipeline_count_at(&body, 2)? <= limit {
        return Ok(true);
    }
    // 被拒的这次不计数。撤不掉也不影响本次拒绝结果。
    let _ = http
        .post(pipeline_endpoint(redis_url))
        .bearer_auth(token)
        .json(&serde_json::json!([["ZREM", key, member.as_str()]]))
        .send()
        .await;
    Ok(false)
}
