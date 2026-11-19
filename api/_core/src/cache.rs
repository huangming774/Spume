//! Upstash Redis 共享缓存（REST 协议）。
//!
//! Vercel 多实例/冷启动场景下，进程内缓存一冷就空、实例间不共享；
//! 配了 Redis 后热点读走共享缓存，未配置时所有函数返回 None，
//! 调用方回退原有进程内缓存，行为不变。
//!
//! 约定：缓存值统一包一层 `{"n":0|1,"b":<body>}`，n=1 表示失败短缓存；
//! key 里的用户输入部分先做字符白名单清洗，避免拼出奇怪命令。

use std::time::Duration;

use serde_json::{json, Value};

use crate::AppState;

const OP_TIMEOUT: Duration = Duration::from_millis(1500);

/// key 片段清洗：只留字母数字和 `.:_-`，其余替换成 `-`。
pub fn sanitize(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn endpoint(state: &AppState) -> String {
    format!("{}/pipeline", state.cfg.redis_url.trim_end_matches('/'))
}

async fn pipeline(state: &AppState, commands: Vec<Vec<String>>) -> Option<Vec<Value>> {
    if !state.cfg.redis_ready() {
        return None;
    }
    let response = state
        .http
        .post(endpoint(state))
        .bearer_auth(&state.cfg.redis_token)
        .timeout(OP_TIMEOUT)
        .json(&commands)
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let rows: Vec<Value> = response.json().await.ok()?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if row.get("error").is_some() {
            return None;
        }
        out.push(row.get("result").cloned().unwrap_or(Value::Null));
    }
    Some(out)
}

/// 读共享缓存。返回 (body, negative)；未配置/未命中/出错返回 None。
pub async fn get(state: &AppState, key: &str) -> Option<(Value, bool)> {
    let rows = pipeline(state, vec![vec!["GET".into(), format!("cache:{key}")]]).await?;
    let raw = rows.first()?.as_str()?;
    let wrapper: Value = serde_json::from_str(raw).ok()?;
    let body = wrapper.get("b")?.clone();
    let negative = wrapper.get("n").and_then(Value::as_i64).unwrap_or(0) == 1;
    Some((body, negative))
}

/// 写共享缓存，带 TTL。失败静默（调用方仍有进程内缓存兜底）。
pub async fn set(state: &AppState, key: &str, body: &Value, negative: bool, ttl: Duration) {
    let wrapper = json!({ "n": if negative { 1 } else { 0 }, "b": body });
    let Ok(raw) = serde_json::to_string(&wrapper) else {
        return;
    };
    let _ = pipeline(
        state,
        vec![vec![
            "SET".into(),
            format!("cache:{key}"),
            raw,
            "EX".into(),
            ttl.as_secs().max(1).to_string(),
        ]],
    )
    .await;
}

/// 删除若干 key（写路径主动失效）。
pub async fn del(state: &AppState, keys: &[&str]) {
    if keys.is_empty() {
        return;
    }
    let mut command = vec!["DEL".to_string()];
    command.extend(keys.iter().map(|key| format!("cache:{key}")));
    let _ = pipeline(state, vec![command]).await;
}

/// 原子自增；首增时设 TTL。返回自增后的值；未配置/出错返回 None。
pub async fn incr(state: &AppState, key: &str, ttl: Duration) -> Option<i64> {
    let rows = pipeline(
        state,
        vec![
            vec!["INCR".into(), key.to_string()],
            vec![
                "EXPIRE".into(),
                key.to_string(),
                ttl.as_secs().max(1).to_string(),
                "NX".into(),
            ],
        ],
    )
    .await?;
    rows.first()?.as_i64()
}

/// 原子自减（不低于 0 由调用方语义保证，这里只做 DECR）。
pub async fn decr(state: &AppState, key: &str) -> Option<i64> {
    let rows = pipeline(state, vec![vec!["DECR".into(), key.to_string()]]).await?;
    rows.first()?.as_i64()
}

/// 读计数；未配置/不存在返回 None。
pub async fn get_int(state: &AppState, key: &str) -> Option<i64> {
    let rows = pipeline(state, vec![vec!["GET".into(), key.to_string()]]).await?;
    let raw = rows.first()?.as_str()?;
    raw.parse::<i64>().ok()
}

/// SET key 1 NX EX ttl：键已存在返回 Some(false)（重复），新建返回 Some(true)。
pub async fn set_nx(state: &AppState, key: &str, ttl: Duration) -> Option<bool> {
    let rows = pipeline(
        state,
        vec![vec![
            "SET".into(),
            key.to_string(),
            "1".into(),
            "NX".into(),
            "EX".into(),
            ttl.as_secs().max(1).to_string(),
        ]],
    )
    .await?;
    Some(rows.first()?.as_str() == Some("OK"))
}

/// 有序集合加分（浏览计数），member 先清洗。
pub async fn zincrby(state: &AppState, key: &str, member: &str, by: f64) {
    let _ = pipeline(
        state,
        vec![vec![
            "ZINCRBY".into(),
            key.to_string(),
            by.to_string(),
            sanitize(member),
        ]],
    )
    .await;
}

/// 取周榜键：ISO 周（东八区），如 views:article:2026-W47。
pub fn week_key(prefix: &str) -> String {
    let now = chrono::Utc::now() + chrono::Duration::hours(8);
    format!("{prefix}:{}", now.format("%G-W%V"))
}

/// 周榜 Top N，返回 (member, score)；未配置/空返回空。
pub async fn ztop(state: &AppState, key: &str, n: usize) -> Vec<(String, f64)> {
    let Some(rows) = pipeline(
        state,
        vec![vec![
            "ZREVRANGE".into(),
            key.to_string(),
            "0".into(),
            n.saturating_sub(1).to_string(),
            "WITHSCORES".into(),
        ]],
    )
    .await
    else {
        return Vec::new();
    };
    let Some(arr) = rows.first().and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut iter = arr.iter();
    while let (Some(member), Some(score)) = (iter.next(), iter.next()) {
        if let (Some(m), Some(s)) = (
            member.as_str(),
            score.as_str().and_then(|s| s.parse::<f64>().ok()),
        ) {
            out.push((m.to_string(), s));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_keeps_safe_chars_only() {
        assert_eq!(sanitize("music:pl:3778678"), "music:pl:3778678");
        assert_eq!(sanitize("a b\nc\"d"), "a-b-c-d");
        assert_eq!(sanitize("1.2.3.4"), "1.2.3.4");
    }
}
