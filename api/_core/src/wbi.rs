//! B 站 WBI 签名。
//!
//! 雷达拉最新投稿时走 `/x/space/wbi/arc/search`。站长自己的投稿列表仍走未签名接口。

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use md5::{Digest, Md5};
use serde_json::Value;

const NAV: &str = "https://api.bilibili.com/x/web-interface/nav";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const CACHE_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const MIXIN_TAB: &[usize] = &[
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    6, 54, 38, 51, 15, 36, 13, 60, 52, 26, 4, 57, 48, 55, 37, 12, 44, 28, 41, 1, 21, 56, 39, 59,
    20, 17, 25, 61, 40, 34, 11, 16, 24, 7, 30, 22, 14, 62, 9, 63, 47, 51, 35,
];

struct CachedKeys {
    img: String,
    sub: String,
    fetched: Instant,
}

static KEYS: LazyLock<Mutex<Option<CachedKeys>>> = LazyLock::new(|| Mutex::new(None));

/// 从 `img_url` 或 `sub_url` 取出文件名，并去掉 `.png`。
pub fn extract_key(raw_url: &str) -> String {
    let raw = raw_url.trim().trim_start_matches("//");
    let path = raw.split('?').next().unwrap_or(raw);
    let base = path.rsplit('/').next().unwrap_or(path);
    base.strip_suffix(".png").unwrap_or(base).to_string()
}

/// 按 B 站固定下标表打乱 `img + sub`，取前 32 个字符。
pub fn mixin_key(orig: &str) -> String {
    let bytes = orig.as_bytes();
    let mut mixed = String::new();
    for index in MIXIN_TAB {
        if let Some(byte) = bytes.get(*index) {
            mixed.push(*byte as char);
        }
    }
    mixed.chars().take(32).collect()
}

/// 给参数加上 `wts` 和 `w_rid`。返回值可以直接接在 `?` 后面。
pub fn signed_query(params: &[(&str, &str)], img: &str, sub: &str, wts: i64) -> String {
    let mut pairs: Vec<(String, String)> = params
        .iter()
        .map(|(key, value)| ((*key).to_string(), sanitize(value)))
        .collect();
    pairs.push(("wts".to_string(), wts.to_string()));
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    let mixin = mixin_key(&format!("{img}{sub}"));
    let canonical = encode_pairs(&pairs);
    let digest = Md5::digest(format!("{canonical}{mixin}").as_bytes());
    pairs.push(("w_rid".to_string(), hex::encode(digest)));
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    encode_pairs(&pairs)
}

/// 读取导航接口里的 WBI 密钥。12 小时内复用上一次的结果。
pub async fn keys(http: &reqwest::Client) -> Result<(String, String), String> {
    if let Some(cached) = fresh() {
        return Ok(cached);
    }
    let response = http
        .get(NAV)
        .timeout(Duration::from_secs(10))
        .header("User-Agent", UA)
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await
        .map_err(|error| format!("WBI 导航请求失败: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("WBI 导航返回 HTTP {}", response.status().as_u16()));
    }
    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("WBI 导航不是 JSON: {error}"))?;
    let code = payload.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 0 {
        return Err(format!("WBI 导航返回 {code}"));
    }
    let img = extract_key(
        payload
            .pointer("/data/wbi_img/img_url")
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    let sub = extract_key(
        payload
            .pointer("/data/wbi_img/sub_url")
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    if img.is_empty() || sub.is_empty() {
        return Err("WBI 导航没有密钥".into());
    }
    store(&img, &sub);
    Ok((img, sub))
}

fn fresh() -> Option<(String, String)> {
    let guard = KEYS.lock().unwrap_or_else(|poison| poison.into_inner());
    let cached = guard.as_ref()?;
    (cached.fetched.elapsed() < CACHE_TTL).then(|| (cached.img.clone(), cached.sub.clone()))
}

fn store(img: &str, sub: &str) {
    let mut guard = KEYS.lock().unwrap_or_else(|poison| poison.into_inner());
    *guard = Some(CachedKeys {
        img: img.to_string(),
        sub: sub.to_string(),
        fetched: Instant::now(),
    });
}

fn sanitize(value: &str) -> String {
    value.chars().filter(|ch| !"!'()*".contains(*ch)).collect()
}

fn encode_pairs(pairs: &[(String, String)]) -> String {
    let mut out = String::new();
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index > 0 {
            out.push('&');
        }
        out.push_str(&query_escape(key));
        out.push('=');
        out.push_str(&query_escape(value));
    }
    out
}

fn query_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMG: &str = "7cd084941338484aae1ad9425b84077c";
    const SUB: &str = "4932caff0ff746eab6f01bf08b70ac45";

    #[test]
    fn extract_key_uses_the_file_stem() {
        assert_eq!(
            extract_key("https://i0.hdslb.com/bfs/wbi/abc123.png?foo=1"),
            "abc123"
        );
        assert_eq!(extract_key("//i0.hdslb.com/bfs/wbi/def.png"), "def");
    }

    #[test]
    fn mixin_is_32_characters() {
        assert_eq!(mixin_key(&format!("{IMG}{SUB}")).len(), 32);
    }

    #[test]
    fn signature_is_stable_and_drops_reserved_characters() {
        let left = signed_query(
            &[
                ("mid", "1061280173"),
                ("pn", "1"),
                ("ps", "1"),
                ("order", "pubdate"),
            ],
            IMG,
            SUB,
            1_700_000_000,
        );
        let right = signed_query(
            &[
                ("order", "pubdate"),
                ("ps", "1"),
                ("pn", "1"),
                ("mid", "1061280173"),
            ],
            IMG,
            SUB,
            1_700_000_000,
        );
        assert_eq!(left, right);
        assert!(left.contains("wts=1700000000"));
        assert!(left.contains("w_rid="));
        let noisy = signed_query(&[("q", "a!b(c)")], IMG, SUB, 1);
        let clean = signed_query(&[("q", "abc")], IMG, SUB, 1);
        assert_eq!(noisy, clean);
    }
}
