use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::config::Config;

type HmacSha256 = Hmac<Sha256>;

pub fn public_url(cfg: &Config, key: &str) -> String {
    format!(
        "{}/{}",
        cfg.r2_public_base.trim_end_matches('/'),
        key.trim_start_matches('/')
    )
}

pub fn presign_put(cfg: &Config, key: &str, expires_secs: u64) -> String {
    signed_put_url(cfg, key, "UNSIGNED-PAYLOAD", expires_secs)
}

pub fn signed_put_url(cfg: &Config, key: &str, payload_hash: &str, expires_secs: u64) -> String {
    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let host = format!("{}.r2.cloudflarestorage.com", cfg.r2_account_id);
    let canonical_uri = format!("/{}/{}", cfg.r2_bucket, encode_key(key));
    let credential = format!("{}/{date_stamp}/auto/s3/aws4_request", cfg.r2_access_key);
    let credential_q = urlencoding(credential.as_str());
    let query = format!(
        "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={credential_q}&X-Amz-Date={amz_date}&X-Amz-Expires={expires_secs}&X-Amz-SignedHeaders=host"
    );
    let canonical = format!("PUT\n{canonical_uri}\n{query}\nhost:{host}\n\nhost\n{payload_hash}");
    let scope = format!("{date_stamp}/auto/s3/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        sha256_hex(canonical.as_bytes())
    );
    let signature = hex::encode(hmac_sha256(
        &signing_key(&cfg.r2_secret_key, &date_stamp),
        &string_to_sign,
    ));
    format!("https://{host}{canonical_uri}?{query}&X-Amz-Signature={signature}")
}

fn signing_key(secret: &str, date: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date);
    let k_region = hmac_sha256(&k_date, "auto");
    let k_service = hmac_sha256(&k_region, "s3");
    hmac_sha256(&k_service, "aws4_request")
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn encode_key(key: &str) -> String {
    key.split('/')
        .map(urlencoding)
        .collect::<Vec<_>>()
        .join("/")
}

fn urlencoding(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presign_is_a_signed_r2_put() {
        let cfg = crate::config::Config::for_test();
        let url = presign_put(&cfg, "uploads/a.webp", 600);
        assert!(
            url.starts_with("https://account.r2.cloudflarestorage.com/taozhiyy/uploads/a.webp?")
        );
        assert!(url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"));
        assert!(url.contains("X-Amz-Signature="));
        let signature = url.split("X-Amz-Signature=").nth(1).unwrap();
        assert_eq!(signature.len(), 64);
    }
}
