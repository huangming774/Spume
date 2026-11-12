//! 安全审计。只保存事件名、结果和截断后的说明，不保存密码、答案或正文。

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use thiserror::Error;

/// 一条审计。所有字符串都借用调用方，只在写入期间使用。
pub struct AuditEvent<'a> {
    /// 事件名，例如 `auth.login`。
    pub event: &'a str,
    /// `success` 或 `failure`。
    pub outcome: &'a str,
    /// 用户 id。未知时为 0。
    pub actor_user_id: i64,
    /// 客户端 IP。只存哈希。
    pub ip: &'a str,
    /// User-Agent。只存哈希。
    pub user_agent: &'a str,
    /// 目标类型。
    pub target_type: &'a str,
    /// 目标 id。
    pub target_id: &'a str,
    /// 短说明。不能包含密码或留言正文。
    pub detail: &'a str,
}

/// 审计写入失败。
#[derive(Debug, Error)]
pub enum AuditError {
    /// 数据库拒绝了插入。
    #[error("安全审计没有写上: {0}")]
    Database(String),
}

/// 把空白收成一个空格，并按字符数截断。
pub fn normalize(value: &str, max_chars: usize) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(max_chars).collect()
}

/// 写入一条审计。IP 和 User-Agent 只留前 16 字节哈希的十六进制。
pub async fn record(pool: &PgPool, event: AuditEvent<'_>) -> Result<(), AuditError> {
    let actor = event.actor_user_id.max(0);
    let detail = normalize(event.detail, 160);
    let payload = serde_json::json!({ "text": detail });
    sqlx::query(
        "INSERT INTO security_audit_logs (action, detail, event, outcome, actor_user_id, ip_hash, user_agent_hash, target_type, target_id, detail_text)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(normalize(event.event, 80))
    .bind(payload)
    .bind(normalize(event.event, 80))
    .bind(normalize(event.outcome, 80))
    .bind(actor)
    .bind(fingerprint("ip", event.ip))
    .bind(fingerprint("ua", event.user_agent))
    .bind(normalize(event.target_type, 80))
    .bind(normalize(event.target_id, 80))
    .bind(&detail)
    .execute(pool)
    .await
    .map_err(|error| AuditError::Database(error.to_string()))?;
    Ok(())
}

fn fingerprint(kind: &str, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    hasher.update([0]);
    hex::encode(&hasher.finalize()[..16])
}
