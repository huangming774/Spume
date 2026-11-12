//! 留言通知邮件。收件人没配置时不发送；配了收件人但 SMTP 不完整则返回错误。

use lettre::message::Message;
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Tokio1Executor;
use sqlx::PgPool;
use thiserror::Error;

use crate::config::Config;

/// 一封留言通知要用的字段。
pub struct GuestbookNotice<'a> {
    /// `guestbook` 或 `friends`。
    pub channel: &'a str,
    /// 留言人。
    pub nickname: &'a str,
    /// 留言正文。
    pub content: &'a str,
    /// 已格式化的时间。
    pub created_at: &'a str,
    /// 这是一条回复。
    pub reply: bool,
}

/// 邮件没有发出。
#[derive(Debug, Error)]
pub enum MailError {
    /// 收件地址或端口不合法。
    #[error("{0}")]
    Invalid(&'static str),
    /// 需要发信，但 SMTP 没配全。
    #[error("SMTP 还没有配置")]
    NotConfigured,
    /// 连接或投递失败。
    #[error("邮件没有发出: {0}")]
    Transport(String),
    /// 查找回复收件人失败。
    #[error("没有找到回复收件人: {0}")]
    Database(String),
}

/// 站长通知的标题。
pub fn owner_subject(channel: &str, reply: bool) -> &'static str {
    match (channel, reply) {
        ("friends", true) => "桃之夭夭：友链留言有新回复",
        ("friends", false) => "桃之夭夭：新的友链留言",
        (_, true) => "桃之夭夭：留言有新回复",
        (_, false) => "桃之夭夭：新的留言",
    }
}

/// 站长通知正文。
pub fn owner_body(notice: &GuestbookNotice<'_>) -> String {
    format!(
        "桃之夭夭有新的留言通知。\n\n留言位置：{}\n留言人：{}\n留言时间：{}\n\n留言内容：\n{}\n\n查看地址：{}",
        channel_name(notice.channel),
        notice.nickname,
        notice.created_at,
        notice.content,
        channel_url(notice.channel)
    )
}

/// 回复通知正文。
pub fn reply_body(notice: &GuestbookNotice<'_>) -> String {
    format!(
        "你在桃之夭夭留下的留言收到了一条回复。\n\n留言位置：{}\n回复时间：{}\n\n回复内容：\n{}\n\n查看地址：{}",
        channel_name(notice.channel),
        notice.created_at,
        notice.content,
        channel_url(notice.channel)
    )
}

/// 给站长发通知。没有 `MAIL_NOTIFY_TO` 时直接成功返回。
pub async fn notify_owner(cfg: &Config, notice: &GuestbookNotice<'_>) -> Result<(), MailError> {
    let to = cfg.mail_notify_to.trim();
    if to.is_empty() {
        return Ok(());
    }
    if !email_ok(to) {
        return Err(MailError::Invalid("站长通知邮箱不正确"));
    }
    send(
        cfg,
        to,
        owner_subject(notice.channel, notice.reply),
        &owner_body(notice),
    )
    .await
}

/// 站长回复留言时通知原作者。没有可通知的邮箱时直接成功返回。
pub async fn notify_reply(
    cfg: &Config,
    pool: &PgPool,
    parent_id: i64,
    notice: &GuestbookNotice<'_>,
) -> Result<(), MailError> {
    if parent_id <= 0 || !notice.reply {
        return Ok(());
    }
    let recipient = reply_recipient(pool, parent_id).await?;
    let Some(recipient) = recipient else {
        return Ok(());
    };
    if recipient.eq_ignore_ascii_case(cfg.smtp_user.trim()) {
        return Ok(());
    }
    send(
        cfg,
        &recipient,
        "桃之夭夭：你的留言收到回复",
        &reply_body(notice),
    )
    .await
}

async fn send(cfg: &Config, to: &str, subject: &str, body: &str) -> Result<(), MailError> {
    if cfg.smtp_host.trim().is_empty()
        || cfg.smtp_user.trim().is_empty()
        || cfg.smtp_pass.is_empty()
        || cfg.smtp_from.trim().is_empty()
    {
        return Err(MailError::NotConfigured);
    }
    let port = cfg
        .smtp_port
        .trim()
        .parse::<u16>()
        .map_err(|_| MailError::Invalid("SMTP 端口不正确"))?;
    let from = cfg.smtp_from.trim();
    if !email_ok(from) || !email_ok(to) {
        return Err(MailError::Invalid("发件人或收件人邮箱不正确"));
    }
    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|_| MailError::Invalid("发件人邮箱不正确"))?,
        )
        .to(to
            .parse()
            .map_err(|_| MailError::Invalid("收件人邮箱不正确"))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|error| MailError::Transport(error.to_string()))?;
    let mailer = transport(
        cfg.smtp_host.trim(),
        port,
        cfg.smtp_user.trim(),
        &cfg.smtp_pass,
    )?;
    mailer
        .send(message)
        .await
        .map_err(|error| MailError::Transport(error.to_string()))?;
    Ok(())
}

fn transport(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, MailError> {
    let builder = if port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|error| MailError::Transport(error.to_string()))?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|error| MailError::Transport(error.to_string()))?
    };
    Ok(builder
        .port(port)
        .credentials(Credentials::new(user.to_string(), pass.to_string()))
        .build())
}

async fn reply_recipient(pool: &PgPool, parent_id: i64) -> Result<Option<String>, MailError> {
    let row = sqlx::query_as::<_, (String, Option<i64>)>(
        "SELECT contact_email, user_id FROM guestbook_messages WHERE id = $1 AND status <> 'deleted'",
    )
    .bind(parent_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| MailError::Database(error.to_string()))?;
    let Some((contact, user_id)) = row else {
        return Ok(None);
    };
    let contact = contact.trim();
    if email_ok(contact) {
        return Ok(Some(contact.to_string()));
    }
    let Some(user_id) = user_id else {
        return Ok(None);
    };
    let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| MailError::Database(error.to_string()))?;
    Ok(email.filter(|value| email_ok(value)))
}

fn channel_name(channel: &str) -> &'static str {
    if channel == "friends" {
        "友链留言"
    } else {
        "留言区"
    }
}

fn channel_url(channel: &str) -> &'static str {
    if channel == "friends" {
        "https://taozhiyy.top/friends"
    } else {
        "https://taozhiyy.top/guestbook"
    }
}

fn email_ok(value: &str) -> bool {
    let Some((name, domain)) = value.split_once('@') else {
        return false;
    };
    !name.is_empty() && domain.contains('.') && !value.contains(char::is_whitespace)
}
