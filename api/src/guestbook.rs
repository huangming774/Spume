//! 留言校验、IP 掩码和地区查询。
//!
//! 地区先读 `guestbook_ip_cache`。缓存没有时请求 ip-api，失败记成「未知地区」。
//! 私网地址不发请求。

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

const CONTENT_MAX: usize = 300;
const NICK_MAX: usize = 12;
const EMAIL_MAX: usize = 254;
const THREAD_DEPTH: usize = 8;

/// 一条已经落库的留言。
#[derive(Clone)]
pub struct StoredMessage {
    /// 留言 id。
    pub id: i64,
    /// 被回复的留言。0 表示顶层。
    pub parent_id: i64,
    /// 显示名。
    pub nickname: String,
    /// 头像地址。可为空。
    pub avatar: String,
    /// 正文。
    pub content: String,
    /// 地区原文。
    pub ip_region: String,
    /// 是否登录用户。
    pub is_login_user: bool,
    /// 是否站长。
    pub is_admin_user: bool,
    /// `visible`、`hidden` 或 `deleted`。
    pub status: String,
    /// 掩码后的 IP。只给站长看。
    pub ip_masked: String,
    /// User-Agent 哈希。只给站长看。
    pub user_agent_hash: String,
    /// 写入时间。
    pub created_at: DateTime<Utc>,
}

/// 留言被拒绝。
#[derive(Debug)]
pub struct GuestError {
    /// 错误码。
    pub code: &'static str,
    /// 给页面的中文说明。
    pub message: &'static str,
}

/// 组装一条留言作者信息。
pub struct ComposeInput<'a> {
    /// `guestbook` 或 `friends`。
    pub channel: &'a str,
    /// 0 表示新楼。
    pub parent_id: i64,
    /// 请求里的昵称。
    pub nickname: &'a str,
    /// 请求里的邮箱。
    pub email: &'a str,
    /// 已登录用户的显示名。
    pub account_name: Option<&'a str>,
    /// 已登录用户的头像。
    pub account_avatar: &'a str,
}

/// 可以写入的作者字段。
#[derive(Debug)]
pub struct Compose {
    /// 最终昵称。
    pub nickname: String,
    /// 最终头像。
    pub avatar: String,
    /// 规范化邮箱。顶层友链留言才有。
    pub email: String,
}

type MessageRow = (
    i64,
    i64,
    String,
    String,
    String,
    String,
    bool,
    bool,
    String,
    String,
    String,
    DateTime<Utc>,
);

/// 环境变量是否打开主留言区。空值和其它值都是关闭。
pub fn posts_enabled(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "enabled"
    )
}

/// 只接受主留言区和友链区。空字符串视为主留言区。
pub fn normalize_channel(raw: &str) -> Option<&'static str> {
    match raw.trim() {
        "" | "guestbook" => Some("guestbook"),
        "friends" => Some("friends"),
        _ => None,
    }
}

/// 页大小。缺省、0、负数和超过 50 都回到 20。
pub fn accepted_page_size(raw: Option<i64>) -> i64 {
    match raw {
        Some(size) if (1..=50).contains(&size) => size,
        _ => 20,
    }
}

/// 正文是否带脚本、事件属性或 HTML 标签。
pub fn looks_unsafe(raw: &str) -> bool {
    let decoded = unescape_html(raw).to_lowercase();
    let compact = compact_unsafe(&decoded);
    if compact.contains("javascript:")
        || compact.contains("vbscript:")
        || compact.contains("data:text/html")
    {
        return true;
    }
    const EVENTS: [&str; 13] = [
        "onabort=",
        "onanimationstart=",
        "onblur=",
        "onchange=",
        "onclick=",
        "onerror=",
        "onfocus=",
        "oninput=",
        "onload=",
        "onmouseenter=",
        "onmouseover=",
        "onpointerenter=",
        "onsubmit=",
    ];
    if EVENTS.iter().any(|event| compact.contains(event)) {
        return true;
    }
    has_html_tag(&decoded)
}

/// 检查已经去掉首尾空白的正文。
pub fn check_content(content: &str) -> Result<(), GuestError> {
    if looks_unsafe(content) {
        return Err(GuestError {
            code: "UNSAFE_CONTENT",
            message: "留言不能包含脚本、HTML 标签或可执行链接",
        });
    }
    if content.is_empty() {
        return Err(GuestError {
            code: "INVALID_CONTENT",
            message: "留言内容不能为空",
        });
    }
    if content.chars().count() > CONTENT_MAX {
        return Err(GuestError {
            code: "INVALID_CONTENT",
            message: "留言最多 300 字哦",
        });
    }
    Ok(())
}

/// 按分区决定昵称、头像和邮箱。
pub fn compose(input: ComposeInput<'_>) -> Result<Compose, GuestError> {
    let friends_root = input.parent_id == 0 && input.channel == "friends";
    let (nickname, email) = if friends_root {
        (accept_nickname(input.nickname)?, accept_email(input.email)?)
    } else if let Some(name) = input.account_name {
        (name.trim().to_string(), String::new())
    } else {
        (accept_nickname(input.nickname)?, String::new())
    };
    if looks_unsafe(&nickname) {
        return Err(GuestError {
            code: "UNSAFE_CONTENT",
            message: "昵称不能包含脚本或 HTML 标签",
        });
    }
    if nickname.is_empty() || nickname.chars().count() > NICK_MAX {
        return Err(GuestError {
            code: "INVALID_NICKNAME",
            message: "昵称最多 12 个字哦",
        });
    }
    Ok(Compose {
        nickname,
        avatar: input.account_avatar.to_string(),
        email,
    })
}

/// 把 IP 收成前两段，IPv6 只留类型。
pub fn mask_ip(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return "*".into();
    }
    let ip = raw
        .parse::<std::net::SocketAddr>()
        .map(|socket| socket.ip().to_string())
        .unwrap_or_else(|_| raw.to_string());
    if ip.contains(':') {
        return "ipv6:*".into();
    }
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.*.*", parts[0], parts[1])
    } else {
        "*".into()
    }
}

/// 回环、私网和链路本地地址。
pub fn is_private_ip(raw: &str) -> bool {
    let raw = raw.trim();
    let ip = raw
        .parse::<std::net::SocketAddr>()
        .map(|socket| socket.ip())
        .or_else(|_| raw.parse::<IpAddr>());
    let Ok(ip) = ip else {
        return false;
    };
    match ip {
        IpAddr::V4(addr) => addr.is_loopback() || ipv4_private(addr) || addr.is_link_local(),
        IpAddr::V6(addr) => addr.is_loopback() || ipv6_local(addr),
    }
}

/// `sha256(kind \\0 value \\0)` 的前 16 字节，十六进制。
pub fn hash_key(kind: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    hasher.update([0]);
    hex::encode(&hasher.finalize()[..16])
}

/// 站长提醒标题。
pub fn notice_title(channel: &str) -> &'static str {
    match channel {
        "friends" => "朋友页消息",
        "guestbook" => "留言板消息",
        _ => "站长提醒",
    }
}

/// 站长提醒里的来源名。
pub fn notice_source(channel: &str) -> &'static str {
    match channel {
        "friends" => "朋友页",
        "guestbook" => "留言板",
        _ => "站长提醒",
    }
}

/// `来源 · 昵称 · 时间`。时间按东八区，当天显示「今天 HH:MM」。
pub fn notice_detail(channel: &str, nickname: &str, created: DateTime<Utc>) -> String {
    let name = nickname.trim();
    let name = if name.is_empty() { "访客" } else { name };
    format!(
        "{} · {name} · {}",
        notice_source(channel),
        china_clock(created)
    )
}

/// 访客角标用的短地区。空和未知显示「访客」，只留最后一段，最多 5 个字。
pub fn compact_region_label(region: &str) -> String {
    let region = region.trim();
    if region.is_empty() || region == "未知地区" {
        return "访客".into();
    }
    let last = region.split_whitespace().last().unwrap_or(region);
    last.chars().take(5).collect()
}

/// 解析 ip-api 的地区。失败返回 None，由调用方记成未知。
pub fn region_from_ip_api(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    if value.get("status").and_then(Value::as_str) != Some("success") {
        return None;
    }
    let region = value
        .get("regionName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let city = value
        .get("city")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !region.is_empty() && !city.is_empty() && region != city {
        return Some(format!("{region} {city}"));
    }
    if !city.is_empty() {
        return Some(city.to_string());
    }
    if !region.is_empty() {
        return Some(region.to_string());
    }
    None
}

/// 查地区。没有数据库时，公网地址返回空字符串，不请求上游。
pub async fn lookup_region(
    pool: Option<&PgPool>,
    http: &reqwest::Client,
    ip: &str,
) -> Result<String, sqlx::Error> {
    if is_private_ip(ip) {
        return Ok("本地".into());
    }
    let Some(pool) = pool else {
        return Ok(String::new());
    };
    let ip_hash = hash_key("ip", ip);
    let cached: Option<String> =
        sqlx::query_scalar("SELECT ip_region FROM guestbook_ip_cache WHERE ip_hash = $1")
            .bind(&ip_hash)
            .fetch_optional(pool)
            .await?;
    if let Some(region) = cached.filter(|region| !region.is_empty()) {
        return Ok(region);
    }
    let region = fetch_region(http, ip)
        .await
        .unwrap_or_else(|| "未知地区".into());
    sqlx::query(
        "INSERT INTO guestbook_ip_cache (ip_hash, ip_region) VALUES ($1, $2)
         ON CONFLICT (ip_hash) DO UPDATE SET ip_region = EXCLUDED.ip_region, updated_at = now()",
    )
    .bind(&ip_hash)
    .bind(&region)
    .execute(pool)
    .await?;
    Ok(region)
}

/// 读出一页顶层留言，以及它们下面的回复树。
pub async fn load_page(
    pool: &PgPool,
    channel: &str,
    page: i64,
    page_size: i64,
    admin: bool,
) -> Result<(Vec<Value>, i64), sqlx::Error> {
    let offset = (page - 1) * page_size;
    let status = if admin {
        "status IN ('visible', 'hidden')"
    } else {
        "status = 'visible'"
    };
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM guestbook_messages WHERE channel = $1 AND parent_id = 0 AND {status}"
    ))
    .bind(channel)
    .fetch_one(pool)
    .await?;
    let roots = fetch_rows(
        pool,
        &format!(
            "SELECT {COLUMNS} FROM guestbook_messages WHERE channel = $1 AND parent_id = 0 AND {status} ORDER BY id DESC LIMIT $2 OFFSET $3"
        ),
        channel,
        Some((page_size, offset)),
        None,
    )
    .await?;
    let replies = load_descendants(pool, channel, &roots, admin).await?;
    Ok((thread_json(roots, replies, admin), total))
}

/// 把顶层和后代排成前端要的回复树。`reveal_client` 为真时带上掩码 IP 和 UA 哈希。
pub fn thread_json(
    roots: Vec<StoredMessage>,
    replies: Vec<StoredMessage>,
    reveal_client: bool,
) -> Vec<Value> {
    let mut children: HashMap<i64, Vec<StoredMessage>> = HashMap::new();
    for reply in replies {
        children.entry(reply.parent_id).or_default().push(reply);
    }
    roots
        .iter()
        .map(|root| node(root, &children, reveal_client, 0, &mut HashSet::new()).0)
        .collect()
}

const COLUMNS: &str = "id, parent_id, nickname, avatar, content, ip_region, is_login_user, is_admin_user, status, ip_masked, user_agent_hash, created_at";

async fn load_descendants(
    pool: &PgPool,
    channel: &str,
    roots: &[StoredMessage],
    admin: bool,
) -> Result<Vec<StoredMessage>, sqlx::Error> {
    let status = if admin {
        "status IN ('visible', 'hidden')"
    } else {
        "status = 'visible'"
    };
    let mut pending: Vec<i64> = roots.iter().map(|item| item.id).collect();
    let mut seen = HashSet::new();
    let mut all = Vec::new();
    for _ in 0..THREAD_DEPTH {
        pending.retain(|id| seen.insert(*id));
        if pending.is_empty() {
            break;
        }
        let rows = fetch_rows(
            pool,
            &format!(
                "SELECT {COLUMNS} FROM guestbook_messages WHERE channel = $1 AND parent_id = ANY($2) AND {status} ORDER BY id ASC"
            ),
            channel,
            None,
            Some(&pending),
        )
        .await?;
        pending = rows.iter().map(|item| item.id).collect();
        all.extend(rows);
    }
    Ok(all)
}

async fn fetch_rows(
    pool: &PgPool,
    sql: &str,
    channel: &str,
    page: Option<(i64, i64)>,
    parents: Option<&[i64]>,
) -> Result<Vec<StoredMessage>, sqlx::Error> {
    let rows = if let Some((limit, offset)) = page {
        sqlx::query_as::<_, MessageRow>(sql)
            .bind(channel)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, MessageRow>(sql)
            .bind(channel)
            .bind(parents.unwrap_or(&[]))
            .fetch_all(pool)
            .await?
    };
    Ok(rows.into_iter().map(stored_from_row).collect())
}

fn stored_from_row(row: MessageRow) -> StoredMessage {
    StoredMessage {
        id: row.0,
        parent_id: row.1,
        nickname: row.2,
        avatar: row.3,
        content: row.4,
        ip_region: row.5,
        is_login_user: row.6,
        is_admin_user: row.7,
        status: row.8,
        ip_masked: row.9,
        user_agent_hash: row.10,
        created_at: row.11,
    }
}

fn node(
    item: &StoredMessage,
    children: &HashMap<i64, Vec<StoredMessage>>,
    reveal_client: bool,
    depth: usize,
    stack: &mut HashSet<i64>,
) -> (Value, i64) {
    if depth >= THREAD_DEPTH || !stack.insert(item.id) {
        return (item_json(item, reveal_client, Vec::new(), 0), 0);
    }
    let mut replies = Vec::new();
    let mut total = 0_i64;
    if let Some(kids) = children.get(&item.id) {
        for child in kids {
            let (value, nested) = node(child, children, reveal_client, depth + 1, stack);
            total += 1 + nested;
            replies.push(value);
        }
    }
    stack.remove(&item.id);
    (item_json(item, reveal_client, replies, total), total)
}

fn item_json(
    item: &StoredMessage,
    reveal_client: bool,
    replies: Vec<Value>,
    reply_count: i64,
) -> Value {
    let region = if item.ip_region.is_empty() {
        "未知地区".to_string()
    } else {
        item.ip_region.clone()
    };
    let mut value = json!({
        "id": item.id,
        "parentId": item.parent_id,
        "nickname": item.nickname,
        "name": item.nickname,
        "avatar": item.avatar,
        "content": item.content,
        "ipRegion": region,
        "isLoginUser": item.is_login_user,
        "isAdminUser": item.is_admin_user,
        "status": item.status,
        "createdAt": item.created_at.to_rfc3339(),
        "replies": replies,
        "replyCount": reply_count
    });
    if reveal_client {
        if !item.ip_masked.is_empty() {
            value["ipMasked"] = json!(item.ip_masked);
        }
        if !item.user_agent_hash.is_empty() {
            value["userAgentHash"] = json!(item.user_agent_hash);
        }
    }
    value
}

fn china_clock(time: DateTime<Utc>) -> String {
    let Some(offset) = chrono::FixedOffset::east_opt(8 * 3600) else {
        return time.to_rfc3339();
    };
    let local = time.with_timezone(&offset);
    let now = Utc::now().with_timezone(&offset);
    if local.date_naive() == now.date_naive() {
        return format!("今天 {}", local.format("%H:%M"));
    }
    if local.year() == now.year() {
        return local.format("%m-%d %H:%M").to_string();
    }
    local.format("%Y-%m-%d %H:%M").to_string()
}

fn accept_nickname(raw: &str) -> Result<String, GuestError> {
    let nickname = raw.trim();
    if looks_unsafe(nickname) {
        return Err(GuestError {
            code: "UNSAFE_CONTENT",
            message: "昵称不能包含脚本或 HTML 标签",
        });
    }
    if nickname.is_empty() {
        return Err(GuestError {
            code: "INVALID_NICKNAME",
            message: "请给自己取个名字呀",
        });
    }
    if nickname.chars().count() > NICK_MAX {
        return Err(GuestError {
            code: "INVALID_NICKNAME",
            message: "昵称最多 12 个字哦",
        });
    }
    Ok(nickname.to_string())
}

fn accept_email(raw: &str) -> Result<String, GuestError> {
    let email = raw.trim().to_ascii_lowercase();
    if email.is_empty() {
        return Err(GuestError {
            code: "INVALID_EMAIL",
            message: "请留下邮箱，方便收到回复通知",
        });
    }
    if !valid_email(&email) {
        return Err(GuestError {
            code: "INVALID_EMAIL",
            message: "请输入有效的邮箱地址",
        });
    }
    Ok(email)
}

fn valid_email(email: &str) -> bool {
    if email.is_empty() || email.chars().count() > EMAIL_MAX {
        return false;
    }
    let mut parts = email.split('@');
    let (Some(local), Some(domain)) = (parts.next(), parts.next()) else {
        return false;
    };
    if parts.next().is_some()
        || local.is_empty()
        || domain.is_empty()
        || local.chars().any(char::is_whitespace)
        || domain.chars().any(char::is_whitespace)
    {
        return false;
    }
    match domain.split_once('.') {
        Some((name, suffix)) => !name.is_empty() && !suffix.is_empty(),
        None => false,
    }
}

fn ipv4_private(addr: Ipv4Addr) -> bool {
    addr.is_private()
}

fn ipv6_local(addr: Ipv6Addr) -> bool {
    addr.is_unicast_link_local() || is_unique_local(addr)
}

fn is_unique_local(addr: Ipv6Addr) -> bool {
    (addr.octets()[0] & 0xfe) == 0xfc
}

async fn fetch_region(http: &reqwest::Client, ip: &str) -> Option<String> {
    let ip = ip.trim();
    if ip.is_empty()
        || ip.len() > 64
        || !ip
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '.' || ch == ':')
    {
        return None;
    }
    let url = format!("http://ip-api.com/json/{ip}?lang=zh-CN&fields=regionName,city,status");
    let response = http
        .get(url)
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().await.ok()?;
    region_from_ip_api(&body)
}

fn unescape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find(';') {
            if let Some(ch) = decode_entity(&after[..end]) {
                out.push(ch);
                rest = &after[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => decode_numeric(entity),
    }
}

fn decode_numeric(entity: &str) -> Option<char> {
    let hex = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"));
    let code = if let Some(digits) = hex {
        u32::from_str_radix(digits, 16).ok()?
    } else {
        entity.strip_prefix('#')?.parse::<u32>().ok()?
    };
    char::from_u32(code).filter(|ch| *ch != '\0')
}

fn compact_unsafe(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_control())
        .collect()
}

fn has_html_tag(value: &str) -> bool {
    let mut rest = value;
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('>') else {
            return false;
        };
        let inside = after[..end].trim().trim_start_matches('/');
        if let Some(first) = inside.chars().next() {
            if first.is_alphabetic() || first == '!' || first == '?' {
                return true;
            }
        }
        rest = &after[end + 1..];
    }
    false
}
