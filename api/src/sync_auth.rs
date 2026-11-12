//! 手动同步的部署口令。口令来自请求头或 `Bearer`，长度不同直接拒绝。

/// 取出调用方带来的口令。`X-Sync-Trigger-Token` 优先于 `Authorization`。
pub fn provided_token(header: Option<&str>, authorization: Option<&str>) -> String {
    if let Some(token) = nonempty(header) {
        return token;
    }
    let Some(authorization) = nonempty(authorization) else {
        return String::new();
    };
    let mut parts = authorization.splitn(2, ' ');
    let scheme = parts.next().unwrap_or("");
    let token = parts.next().unwrap_or("").trim();
    if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() {
        token.to_string()
    } else {
        String::new()
    }
}

/// 常量时间比较。期望值为空，或两边长度不同，都算不匹配。
pub fn token_matches(expected: &str, provided: &str) -> bool {
    let expected = expected.trim();
    if expected.is_empty() || provided.len() != expected.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left, right) in expected.bytes().zip(provided.bytes()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn nonempty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
}
