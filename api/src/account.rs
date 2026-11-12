//! 注册、登录和站长账号的输入规则，与原来的账号接口一致。

use sqlx::PgPool;

/// 邮箱要有本地部分、域名里的点，并且不超过 254 个字符。
pub fn valid_email(email: &str) -> bool {
    if email.is_empty() || email.chars().count() > 254 || email.contains(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.contains('@') {
        return false;
    }
    let Some((host, rest)) = domain.split_once('.') else {
        return false;
    };
    !host.is_empty() && !rest.is_empty()
}

/// 密码至少 8 个字符，字节数不超过 bcrypt 的 72。
pub fn password_message(password: &str) -> Option<&'static str> {
    if password.chars().count() < 8 {
        Some("密码至少 8 位")
    } else if password.len() > 72 {
        Some("密码过长")
    } else {
        None
    }
}

/// 昵称去掉空白后要有内容，最多 12 个字。
pub fn display_name_message(name: &str) -> Option<&'static str> {
    let name = name.trim();
    if name.is_empty() {
        Some("昵称不能为空")
    } else if name.chars().count() > 12 {
        Some("昵称最多 12 个字")
    } else {
        None
    }
}

/// 站长账号播种结果。
pub enum OwnerSeed {
    /// 库里已经是站长。
    Ready,
    /// 没有 `AUTH_OWNER_PASSWORD`，还不能登录。
    PasswordMissing,
    /// 这次新建了站长账号。
    Created,
}

/// 保证 `email` 是站长。账号不存在且口令可用时才创建。
pub async fn ensure_owner(pool: &PgPool, email: &str) -> Result<OwnerSeed, sqlx::Error> {
    let email = email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(OwnerSeed::Ready);
    }
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await?;
    if let Some(id) = existing {
        sqlx::query("UPDATE users SET is_owner = true WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(OwnerSeed::Ready);
    }
    let password = std::env::var("AUTH_OWNER_PASSWORD").unwrap_or_default();
    let password = password.trim();
    if password_message(password).is_some() {
        return Ok(OwnerSeed::PasswordMissing);
    }
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|error| sqlx::Error::Protocol(format!("owner password hash failed: {error}")))?;
    sqlx::query("INSERT INTO users (email, password_hash, is_owner) VALUES ($1, $2, true)")
        .bind(email)
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(OwnerSeed::Created)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_and_password_follow_the_go_rules() {
        assert!(valid_email("owner@example.com"));
        assert!(!valid_email("owner@example"));
        assert!(!valid_email("a@.com"));
        assert!(!valid_email("a@b.c d"));
        assert_eq!(password_message("short"), Some("密码至少 8 位"));
        assert_eq!(password_message(&"a".repeat(73)), Some("密码过长"));
        assert_eq!(password_message("12345678"), None);
        assert_eq!(display_name_message("  "), Some("昵称不能为空"));
        assert_eq!(
            display_name_message(&"名".repeat(13)),
            Some("昵称最多 12 个字")
        );
    }
}
