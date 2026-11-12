use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};

use crate::AppState;

const PAGE_SIZE: i64 = 30;
const MAX_PAGES: i64 = 100;
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug)]
struct SyncError {
    reason: &'static str,
    code: Option<i32>,
    detail: String,
}

struct OwnerVideo {
    bvid: String,
    title: String,
    description: String,
    cover_remote: String,
    published_at: DateTime<Utc>,
    duration: String,
    author: String,
}

pub async fn sync_owner(state: &AppState) -> Result<usize, String> {
    let Some(pool) = state.db.as_ref() else {
        return Err("数据库还没有连上".into());
    };
    let uid = state.cfg.bilibili_uid.clone();
    if !valid_uid(&uid) {
        return Err("B站 UID 配置不正确".into());
    }
    write_status(pool, &uid, "syncing", "", None, false).await;
    match fetch_all(state, &uid).await {
        Ok((author, videos)) => {
            if let Err(error) = save_videos(pool, &uid, &author, &videos).await {
                tracing::warn!(%error, "bili snapshot was not saved");
                write_status(pool, &uid, "error", "cache_error", None, false).await;
                return Err("投稿缓存暂不可用".into());
            }
            Ok(videos.len())
        }
        Err(error) => {
            tracing::warn!(reason = error.reason, code = error.code, detail = %error.detail, "bili sync failed");
            write_status(pool, &uid, "error", error.reason, error.code, false).await;
            Err(public_reason(error.reason).into())
        }
    }
}

pub async fn owner_feed(state: &AppState) -> Result<Value, String> {
    let uid = state.cfg.bilibili_uid.clone();
    let profile = format!("https://space.bilibili.com/{uid}");
    let Some(pool) = state.db.as_ref() else {
        return Ok(empty_feed(&uid, &profile));
    };
    let rows = sqlx::query_as::<_, (String, String, String, String, String, DateTime<Utc>, String, DateTime<Utc>)>(
        "SELECT bvid, title, description, cover, cover_remote, published_at, duration, synced_at FROM bilibili_videos ORDER BY published_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| format!("投稿列表没有读出来: {error}"))?;
    let sync = sqlx::query_as::<_, (String, String, Option<DateTime<Utc>>, Option<i32>, Option<DateTime<Utc>>, String)>(
        "SELECT state, reason, attempted_at, upstream_code, synced_at, owner_name FROM bili_sync WHERE uid = $1",
    )
    .bind(&uid)
    .fetch_optional(pool)
    .await
    .map_err(|error| format!("投稿同步状态没有读出来: {error}"))?;
    let ready = !rows.is_empty();
    let synced_at = rows
        .iter()
        .map(|row| row.7)
        .max()
        .map(|value| value.to_rfc3339())
        .unwrap_or_default();
    let name = sync
        .as_ref()
        .map(|row| row.5.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "taozhiyo".into());
    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(bvid, title, description, cover, remote, published, duration, _)| {
                json!({
                    "bvid": bvid,
                    "title": title,
                    "description": description,
                    "cover": public_cover(&bvid, &cover, &remote),
                    "publishedAt": published.to_rfc3339(),
                    "duration": duration
                })
            },
        )
        .collect();
    let (state_name, reason, attempted_at, upstream) = match sync {
        Some((state_name, reason, attempted, code, _, _)) => (
            state_name,
            reason,
            attempted
                .map(|value| value.to_rfc3339())
                .unwrap_or_default(),
            code,
        ),
        None => (
            if ready { "ok" } else { "pending" }.to_string(),
            String::new(),
            String::new(),
            None,
        ),
    };
    let mut sync_json = json!({"state": state_name, "reason": reason, "attemptedAt": attempted_at});
    if let Some(code) = upstream {
        sync_json["upstreamCode"] = json!(code);
    }
    Ok(json!({
        "owner": {"uid": uid, "name": name, "profileUrl": profile},
        "items": items,
        "ready": ready,
        "syncedAt": if ready { synced_at } else { String::new() },
        "sync": sync_json
    }))
}

pub fn bvid_from_cover_name(name: &str) -> Option<&str> {
    let bvid = name.strip_prefix("owner_")?.strip_suffix(".jpg")?;
    valid_bvid(bvid).then_some(bvid)
}

pub async fn cover_source(state: &AppState, name: &str) -> Result<Option<String>, String> {
    let Some(bvid) = bvid_from_cover_name(name) else {
        return Ok(None);
    };
    let Some(pool) = state.db.as_ref() else {
        return Ok(None);
    };
    let remote: Option<String> =
        sqlx::query_scalar("SELECT cover_remote FROM bilibili_videos WHERE bvid = $1")
            .bind(bvid)
            .fetch_optional(pool)
            .await
            .map_err(|error| format!("封面地址没有读出来: {error}"))?;
    Ok(remote.and_then(|value| remote_cover(&value)))
}

pub async fn fetch_cover(state: &AppState, remote: &str) -> Result<Vec<u8>, ()> {
    let response = state
        .http
        .get(remote)
        .timeout(std::time::Duration::from_secs(10))
        .header("User-Agent", UA)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|_| ())?;
    if !response.status().is_success() {
        return Err(());
    }
    let bytes = response.bytes().await.map_err(|_| ())?;
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err(());
    }
    Ok(bytes.to_vec())
}

fn empty_feed(uid: &str, profile: &str) -> Value {
    json!({
        "owner": {"uid": uid, "name": "taozhiyo", "profileUrl": profile},
        "items": [],
        "ready": false,
        "syncedAt": "",
        "sync": {"state": "pending", "reason": "", "attemptedAt": ""}
    })
}

fn public_cover(bvid: &str, cover: &str, remote: &str) -> String {
    if !remote.is_empty() {
        return format!("/api/v1/acg/image/owner_{bvid}.jpg");
    }
    cover.to_string()
}

async fn fetch_all(state: &AppState, uid: &str) -> Result<(String, Vec<OwnerVideo>), SyncError> {
    let mut videos = Vec::new();
    let mut total: Option<i64> = None;
    let mut author = String::new();
    for page in 1..=MAX_PAGES {
        let payload = fetch_page(state, uid, page).await?;
        let (page_total, page_author, rows) = parse_page(uid, page, &payload)?;
        if total.is_some_and(|seen| seen != page_total)
            || !(0..=MAX_PAGES * PAGE_SIZE).contains(&page_total)
        {
            return Err(invalid("owner archive count changed"));
        }
        total = Some(page_total);
        if !page_author.is_empty() {
            if !author.is_empty() && author != page_author {
                return Err(invalid("owner archive author changed"));
            }
            author = page_author;
        }
        let expected = (page_total - videos.len() as i64).clamp(0, PAGE_SIZE);
        if rows.len() as i64 != expected {
            return Err(invalid("owner archive page length mismatch"));
        }
        videos.extend(rows);
        if videos.len() as i64 == page_total {
            return Ok((author, videos));
        }
    }
    Err(invalid("owner archive exceeded page limit"))
}

async fn fetch_page(state: &AppState, uid: &str, page: i64) -> Result<Value, SyncError> {
    let url = format!("https://api.bilibili.com/x/space/arc/search?mid={uid}&ps={PAGE_SIZE}&pn={page}&order=pubdate");
    let response = state
        .http
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .header("User-Agent", UA)
        .header("Referer", format!("https://space.bilibili.com/{uid}/"))
        .send()
        .await
        .map_err(|error| SyncError {
            reason: "network_error",
            code: None,
            detail: error.to_string(),
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(SyncError {
            reason: "upstream_rejected",
            code: Some(status.as_u16() as i32),
            detail: status.to_string(),
        });
    }
    response
        .json::<Value>()
        .await
        .map_err(|error| invalid(&error.to_string()))
}

fn parse_page(
    uid: &str,
    page: i64,
    body: &Value,
) -> Result<(i64, String, Vec<OwnerVideo>), SyncError> {
    let code = body
        .get("code")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid("missing code"))?;
    if code != 0 {
        return Err(SyncError {
            reason: "upstream_rejected",
            code: Some(code as i32),
            detail: format!("code {code}"),
        });
    }
    let data = body.get("data").ok_or_else(|| invalid("missing data"))?;
    let page_info = data.get("page").ok_or_else(|| invalid("missing page"))?;
    let total = page_info
        .get("count")
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid("missing count"))?;
    let pn = page_info.get("pn").and_then(Value::as_i64).unwrap_or(0);
    let ps = page_info.get("ps").and_then(Value::as_i64).unwrap_or(0);
    if pn != page || ps != PAGE_SIZE {
        return Err(invalid("pagination mismatch"));
    }
    let list = data
        .get("list")
        .and_then(|item| item.get("vlist"))
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("missing vlist"))?;
    let mut author = String::new();
    let mut videos = Vec::new();
    for row in list {
        let video = parse_video(uid, row)?;
        if !author.is_empty() && author != video.author {
            return Err(invalid("mixed authors"));
        }
        author = video.author.clone();
        videos.push(video);
    }
    Ok((total, author, videos))
}

fn parse_video(uid: &str, row: &Value) -> Result<OwnerVideo, SyncError> {
    let bvid = row.get("bvid").and_then(Value::as_str).unwrap_or("");
    let mid = row
        .get("mid")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_default();
    let title = row
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let description = row
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_string);
    let cover = remote_cover(row.get("pic").and_then(Value::as_str).unwrap_or(""));
    let created = row.get("created").and_then(Value::as_i64).unwrap_or(0);
    let duration = row.get("length").and_then(Value::as_str).unwrap_or("");
    let author = row
        .get("author")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if mid != uid
        || !valid_bvid(bvid)
        || title.is_empty()
        || description.is_none()
        || author.is_empty()
        || !valid_duration(duration)
        || cover.is_none()
        || !(1..=253_402_300_799).contains(&created)
    {
        return Err(invalid("invalid owner video"));
    }
    Ok(OwnerVideo {
        bvid: bvid.to_string(),
        title,
        description: description.unwrap_or_default(),
        cover_remote: cover.unwrap_or_default(),
        published_at: Utc
            .timestamp_opt(created, 0)
            .single()
            .ok_or_else(|| invalid("invalid publish time"))?,
        duration: duration.to_string(),
        author,
    })
}

async fn save_videos(
    pool: &sqlx::PgPool,
    uid: &str,
    author: &str,
    videos: &[OwnerVideo],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut ids = Vec::with_capacity(videos.len());
    for video in videos {
        ids.push(video.bvid.clone());
        sqlx::query(
            "INSERT INTO bilibili_videos (bvid, title, description, cover, cover_remote, published_at, duration, synced_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, now())
             ON CONFLICT (bvid) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description, cover = EXCLUDED.cover, cover_remote = EXCLUDED.cover_remote, published_at = EXCLUDED.published_at, duration = EXCLUDED.duration, synced_at = now()",
        )
        .bind(&video.bvid)
        .bind(&video.title)
        .bind(&video.description)
        .bind(format!("/api/v1/acg/image/owner_{}.jpg", video.bvid))
        .bind(&video.cover_remote)
        .bind(video.published_at)
        .bind(&video.duration)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM bilibili_videos WHERE NOT (bvid = ANY($1))")
        .bind(&ids)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO bili_sync (uid, state, reason, attempted_at, upstream_code, synced_at, owner_name)
         VALUES ($1, 'ok', '', now(), NULL, now(), $2)
         ON CONFLICT (uid) DO UPDATE SET state = 'ok', reason = '', upstream_code = NULL, synced_at = now(), owner_name = EXCLUDED.owner_name, attempted_at = now()",
    )
    .bind(uid)
    .bind(author)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn write_status(
    pool: &sqlx::PgPool,
    uid: &str,
    state: &str,
    reason: &str,
    code: Option<i32>,
    synced: bool,
) {
    let _ = sqlx::query(
        "INSERT INTO bili_sync (uid, state, reason, attempted_at, upstream_code, synced_at, owner_name)
         VALUES ($1, $2, $3, now(), $4, CASE WHEN $5 THEN now() ELSE NULL END, '')
         ON CONFLICT (uid) DO UPDATE SET state = EXCLUDED.state, reason = EXCLUDED.reason, attempted_at = now(), upstream_code = EXCLUDED.upstream_code,
           synced_at = CASE WHEN $5 THEN now() ELSE bili_sync.synced_at END",
    )
    .bind(uid)
    .bind(state)
    .bind(reason)
    .bind(code)
    .bind(synced)
    .execute(pool)
    .await;
}

fn public_reason(reason: &str) -> &'static str {
    match reason {
        "network_error" => "投稿同步网络暂不可用",
        "upstream_rejected" => "B站投稿接口暂不可用",
        "cache_error" => "投稿缓存暂不可用",
        _ => "投稿数据校验未通过",
    }
}

fn invalid(detail: &str) -> SyncError {
    SyncError {
        reason: "invalid_response",
        code: None,
        detail: detail.to_string(),
    }
}

fn valid_uid(uid: &str) -> bool {
    let mut chars = uid.chars();
    matches!(chars.next(), Some('1'..='9'))
        && chars.all(|ch| ch.is_ascii_digit())
        && uid.len() <= 19
        && uid.parse::<i64>().is_ok()
}

fn valid_bvid(value: &str) -> bool {
    value
        .strip_prefix("BV")
        .is_some_and(|rest| rest.len() == 10 && rest.chars().all(|ch| ch.is_ascii_alphanumeric()))
}

fn valid_duration(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    let clock = |part: &str| {
        part.len() == 2
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && part.parse::<u8>().ok().is_some_and(|number| number < 60)
    };
    let head = |part: &str| {
        (1..=4).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_digit())
    };
    match parts.as_slice() {
        [mins, secs] => head(mins) && clock(secs),
        [hours, mins, secs] => head(hours) && clock(mins) && clock(secs),
        _ => false,
    }
}

/// 只接受 `https://i数字.hdslb.com/...` 封面。其它地址返回 None。
pub fn remote_cover(raw: &str) -> Option<String> {
    normalize_cover(raw)
}

fn normalize_cover(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 4096 {
        return None;
    }
    let absolute = if let Some(rest) = raw.strip_prefix("//") {
        format!("https://{rest}")
    } else if let Some(rest) = raw.strip_prefix("http://") {
        format!("https://{rest}")
    } else {
        raw.to_string()
    };
    let rest = absolute.strip_prefix("https://")?;
    if rest.contains('@') || rest.contains('#') {
        return None;
    }
    let (host_port, path) = rest.split_once('/')?;
    if path.is_empty() {
        return None;
    }
    let (host, port) = host_port
        .split_once(':')
        .map(|(host, port)| (host, Some(port)))
        .unwrap_or((host_port, None));
    if port.is_some_and(|port| port != "443") || !hdslb_host(host) {
        return None;
    }
    Some(absolute)
}

fn hdslb_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    let Some(rest) = host
        .strip_prefix('i')
        .and_then(|value| value.strip_suffix(".hdslb.com"))
    else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        json!({
            "code": 0,
            "data": {
                "page": {"pn": 1, "ps": 30, "count": 1},
                "list": {"vlist": [{
                    "bvid": "BV1jK7W6MEAj",
                    "mid": 1061280173,
                    "title": "星空",
                    "description": "封面",
                    "pic": "//i0.hdslb.com/bfs/archive/cover.jpg",
                    "created": 1730419200,
                    "length": "03:21",
                    "author": "taozhiyo"
                }]}
            }
        })
    }

    #[test]
    fn archive_page_becomes_a_proxy_cover() {
        let (_total, author, videos) = parse_page("1061280173", 1, &sample()).unwrap();
        assert_eq!(author, "taozhiyo");
        assert_eq!(
            videos[0].cover_remote,
            "https://i0.hdslb.com/bfs/archive/cover.jpg"
        );
        assert_eq!(
            public_cover(&videos[0].bvid, "", &videos[0].cover_remote),
            "/api/v1/acg/image/owner_BV1jK7W6MEAj.jpg"
        );
        assert_eq!(
            bvid_from_cover_name("owner_BV1jK7W6MEAj.jpg"),
            Some("BV1jK7W6MEAj")
        );
    }

    #[test]
    fn foreign_cover_host_is_rejected() {
        let mut body = sample();
        body["data"]["list"]["vlist"][0]["pic"] = json!("https://evil.example/cover.jpg");
        assert!(parse_page("1061280173", 1, &body).is_err());
        assert!(normalize_cover("https://i0.hdslb.com.evil/cover.jpg").is_none());
    }
}
