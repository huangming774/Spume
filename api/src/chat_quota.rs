//! 聊天额度。游客每天 10 次，登录用户每天 50 次，两次提问至少间隔 5 秒。
//!
//! 游客身份是 `guest:` 加上 `sha256(ip|ua)` 的前 8 字节。站长不占每日次数，但仍受 5 秒间隔限制。

use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

/// 游客每日次数。
pub const GUEST_DAILY_LIMIT: i64 = 10;
/// 登录用户每日次数。
pub const USER_DAILY_LIMIT: i64 = 50;

/// 距离上次提问是否已经满 5 秒。没有上次记录时可以提问。
pub fn pace_open(last: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match last {
        None => true,
        Some(last) => now.signed_duration_since(last) >= Duration::seconds(5),
    }
}

/// 游客额度键。同一 IP 和 User-Agent 共用一天的次数。
pub fn guest_key(ip: &str, user_agent: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{ip}|{user_agent}"));
    format!("guest:{}", hex::encode(&hasher.finalize()[..8]))
}
