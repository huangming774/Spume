//! QStash `Upstash-Signature` 校验。
//!
//! JWT 使用 HS256。签名密钥是控制台里的原始字符串，不做 base64 解码。
//! `body` 声明是请求原始字节的 SHA-256，再用无填充 base64url 编码。

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// 允许调用方时钟比 QStash 快或慢 5 秒。QStash 令牌本身大约 5 分钟过期。
pub const CLOCK_TOLERANCE_SECS: i64 = 5;

/// 一次校验需要的材料。所有字符串都只在这次调用期间借用。
pub struct VerifyInput<'a> {
    /// 当前签名密钥。空字符串表示没有配置。
    pub current_key: &'a str,
    /// 轮换中的下一把密钥。空字符串表示不尝试第二把。
    pub next_key: &'a str,
    /// `Upstash-Signature` 头。可以是逗号分隔的多个 JWT，任一把密钥验过即可。
    pub signature: &'a str,
    /// 未经重新序列化的原始请求体。
    pub body: &'a [u8],
    /// 发布到 QStash 的完整目标 URL，包含路径和查询串。
    pub url: &'a str,
    /// 当前 Unix 秒。测试传入固定值，请求处理传入 `Utc::now().timestamp()`。
    pub now_unix: i64,
}

/// 签名校验失败的原因。对外只需要展示通用文案，具体变体留给测试。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QStashError {
    /// 头里没有可解析的 JWT。
    #[error("调度签名格式不正确")]
    Malformed,
    /// 两把密钥都无法还原这个签名。
    #[error("调度签名没有通过")]
    BadSignature,
    /// `iss` 不是 Upstash。
    #[error("调度签发方不正确")]
    Issuer,
    /// `sub` 和本次目标 URL 不一致。
    #[error("调度目标地址不一致")]
    Subject,
    /// `exp` 或 `nbf` 不在允许的时间窗里。
    #[error("调度签名已过期或尚未生效")]
    Time,
    /// 请求体哈希和 `body` 声明不一致。
    #[error("调度请求体已被改动")]
    Body,
}

/// 校验 QStash 回调。成功时不返回声明内容，调用方只需要知道可以执行任务。
pub fn verify(input: &VerifyInput<'_>) -> Result<(), QStashError> {
    if input.current_key.is_empty() {
        return Err(QStashError::BadSignature);
    }
    let digest = body_hash(input.body);
    let mut last = QStashError::Malformed;
    for token in input.signature.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        match verify_token(token, input.current_key, input.url, input.now_unix, &digest) {
            Ok(()) => return Ok(()),
            Err(QStashError::BadSignature) if !input.next_key.is_empty() => {
                match verify_token(token, input.next_key, input.url, input.now_unix, &digest) {
                    Ok(()) => return Ok(()),
                    Err(error) => last = error,
                }
            }
            Err(error) => last = error,
        }
    }
    Err(last)
}

fn verify_token(
    token: &str,
    key: &str,
    url: &str,
    now_unix: i64,
    digest: &str,
) -> Result<(), QStashError> {
    let (header, payload, signature) = split_token(token)?;
    let header_json = decode_part(header)?;
    let header_value: serde_json::Value =
        serde_json::from_slice(&header_json).map_err(|_| QStashError::Malformed)?;
    if header_value.get("alg").and_then(serde_json::Value::as_str) != Some("HS256") {
        return Err(QStashError::Malformed);
    }
    check_mac(header, payload, signature, key)?;
    let payload_json = decode_part(payload)?;
    let claims: serde_json::Value =
        serde_json::from_slice(&payload_json).map_err(|_| QStashError::Malformed)?;
    if claims.get("iss").and_then(serde_json::Value::as_str) != Some("Upstash") {
        return Err(QStashError::Issuer);
    }
    if claims.get("sub").and_then(serde_json::Value::as_str) != Some(url) {
        return Err(QStashError::Subject);
    }
    let exp = claims
        .get("exp")
        .and_then(serde_json::Value::as_i64)
        .ok_or(QStashError::Time)?;
    let nbf = claims
        .get("nbf")
        .and_then(serde_json::Value::as_i64)
        .ok_or(QStashError::Time)?;
    if now_unix > exp + CLOCK_TOLERANCE_SECS || now_unix + CLOCK_TOLERANCE_SECS < nbf {
        return Err(QStashError::Time);
    }
    let claimed = claims
        .get("body")
        .and_then(serde_json::Value::as_str)
        .ok_or(QStashError::Body)?;
    if !constant_eq(claimed.trim_end_matches('=').as_bytes(), digest.as_bytes()) {
        return Err(QStashError::Body);
    }
    Ok(())
}

fn split_token(token: &str) -> Result<(&str, &str, &str), QStashError> {
    let mut parts = token.split('.');
    let header = parts.next().ok_or(QStashError::Malformed)?;
    let payload = parts.next().ok_or(QStashError::Malformed)?;
    let signature = parts.next().ok_or(QStashError::Malformed)?;
    if parts.next().is_some() || header.is_empty() || payload.is_empty() || signature.is_empty() {
        return Err(QStashError::Malformed);
    }
    Ok((header, payload, signature))
}

fn decode_part(part: &str) -> Result<Vec<u8>, QStashError> {
    URL_SAFE_NO_PAD
        .decode(part)
        .map_err(|_| QStashError::Malformed)
}

fn check_mac(header: &str, payload: &str, signature: &str, key: &str) -> Result<(), QStashError> {
    let expected = decode_part(signature)?;
    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| QStashError::BadSignature)?;
    mac.update(header.as_bytes());
    mac.update(b".");
    mac.update(payload.as_bytes());
    mac.verify_slice(&expected)
        .map_err(|_| QStashError::BadSignature)
}

fn body_hash(body: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(body))
}

fn constant_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}
