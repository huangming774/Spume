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
