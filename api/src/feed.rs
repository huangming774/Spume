//! 番剧和雷达同步。
//!
//! 雷达默认跟踪 Go 版写死的四位 UP。`RADAR_CREATORS` 用 `uid:名字` 逗号分隔来替换。
//! 封面只保留 `i*.hdslb.com` 的地址。列表接口把它换成 `/api/v1/acg/image/` 下的同源路径。

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::bili::remote_cover;
use crate::AppState;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

const DEFAULT_CREATORS: &[(&str, &str)] = &[
    ("517327498", "罗翔说刑法"),
    ("431073645", "你的影月月"),
    ("37974444", "黑马程序员"),
    ("398915225", "挪威的月亮"),
];

/// 一位要跟踪的 UP。名字和 uid 是这次同步自己的数据。
pub struct Creator {
    /// B 站数字 uid。
    pub uid: String,
    /// 显示名，不含 `UP ·` 前缀。
    pub name: String,
}

/// 雷达同步结果。失败的 UP 留在 `failed` 里，不把已经写进数据库的条目回滚。
pub struct RadarReport {
    /// 成功写入的 UP 数。
    pub saved: usize,
    /// 每个失败 UP 的原因。
    pub failed: Vec<String>,
}

/// 一条番剧。
pub struct BangumiItem {
    /// `s` 加 season id。
    pub id: String,
    /// 番剧标题。
    pub title: String,
    /// 已看集数。
    pub watched: i32,
    /// 总集数。
    pub total: i32,
    /// 最新一集。
    pub latest_episode: i32,
    /// hdslb 封面，没有时为空。
    pub cover: String,
    /// 播放页。不是 http(s) 时为空。
    pub link: String,
}

/// 同步或解析失败。
#[derive(Debug, Error)]
pub enum FeedError {
    /// 没有数据库。
    #[error("数据库还没有连上")]
    NoDatabase,
    /// 配置或响应结构不对。
    #[error("{0}")]
    Invalid(&'static str),
    /// B 站拒绝或没有内容。
    #[error("{0}")]
    Upstream(String),
    /// 数据库写入失败。
    #[error("{0}")]
    Database(String),
}

/// 解析雷达 UP 配置。空字符串使用内置四位 UP。
pub fn creators(raw: &str) -> Result<Vec<Creator>, FeedError> {
    if raw.trim().is_empty() {
        return Ok(DEFAULT_CREATORS
            .iter()
            .map(|(uid, name)| Creator {
                uid: (*uid).to_string(),
                name: (*name).to_string(),
            })
            .collect());
    }
    let mut parsed = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((uid, name)) = part.split_once(':') else {
            return Err(FeedError::Invalid("雷达 UP 要写成 uid:名字"));
        };
        let uid = uid.trim();
        let name = name.trim();
        if !valid_uid(uid) || name.is_empty() {
            return Err(FeedError::Invalid("雷达 UP 的 uid 或名字无效"));
        }
        parsed.push(Creator {
            uid: uid.to_string(),
            name: name.to_string(),
        });
    }
    if parsed.is_empty() {
        return Err(FeedError::Invalid("雷达 UP 列表是空的"));
    }
    Ok(parsed)
}

/// 拉取配置里的 UP 最新投稿并写入 `radar_feed`。
pub async fn sync_radar(state: &AppState, now: DateTime<Utc>) -> Result<RadarReport, FeedError> {
    let pool = state.db.as_ref().ok_or(FeedError::NoDatabase)?;
    let list = creators(&state.cfg.radar_creators)?;
    let mut saved = 0usize;
    let mut failed = Vec::new();
    for creator in &list {
        match fetch_latest(state, &creator.uid).await {
            Ok(post) => match save_radar(pool, creator, &post, now).await {
                Ok(()) => saved += 1,
                Err(error) => failed.push(format!("{}: {error}", creator.name)),
            },
            Err(error) => failed.push(format!("{}: {error}", creator.name)),
        }
    }
    if saved > 0 {
        sqlx::query("DELETE FROM radar_feed WHERE id LIKE 'seed_%'")
            .execute(pool)
            .await
            .map_err(|error| {
                FeedError::Database(format!("已写入 {saved} 条，清理示例数据失败: {error}"))
            })?;
    }
    if saved == 0 && !failed.is_empty() {
        return Err(FeedError::Upstream(failed.join("; ")));
    }
    Ok(RadarReport { saved, failed })
}

/// 拉取站长追番列表并整表替换。空列表保留原来的缓存。
pub async fn sync_bangumi(state: &AppState) -> Result<usize, FeedError> {
    let pool = state.db.as_ref().ok_or(FeedError::NoDatabase)?;
    let rows = fetch_bangumi(state).await?;
    if rows.is_empty() {
        return Ok(0);
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| FeedError::Database(error.to_string()))?;
    sqlx::query("DELETE FROM bangumi_items")
        .execute(&mut *tx)
        .await
        .map_err(|error| FeedError::Database(error.to_string()))?;
    for row in &rows {
        sqlx::query(
            "INSERT INTO bangumi_items (id, title, watched, total, latest_episode, cover_path, link_url) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&row.id)
        .bind(&row.title)
        .bind(row.watched)
        .bind(row.total)
        .bind(row.latest_episode)
        .bind(&row.cover)
        .bind(&row.link)
        .execute(&mut *tx)
        .await
        .map_err(|error| FeedError::Database(error.to_string()))?;
    }
    tx.commit()
        .await
        .map_err(|error| FeedError::Database(error.to_string()))?;
    Ok(rows.len())
}

/// 表是空的时候写入 Go 版那组占位番剧和雷达。同步到真实数据后会被替换。
pub async fn seed_defaults(pool: &sqlx::PgPool) -> Result<(bool, bool), sqlx::Error> {
    let mut bangumi_seeded = false;
    let mut radar_seeded = false;
    let bangumi_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bangumi_items").fetch_one(pool).await?;
    if bangumi_count == 0 {
        for (id, title, watched, total, latest) in [
            ("b1", "A Certain Scientific Railgun", 12, 24, 24),
            ("b2", "Healing Sketchbook", 8, 12, 10),
            ("b3", "Chronicles of Creation", 3, 13, 7),
            ("b4", "Reimu Garden", 6, 12, 9),
        ] {
            sqlx::query(
                "INSERT INTO bangumi_items (id, title, watched, total, latest_episode) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(title)
            .bind(watched)
            .bind(total)
            .bind(latest)
            .execute(pool)
            .await?;
        }
        bangumi_seeded = true;
    }
    let radar_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM radar_feed").fetch_one(pool).await?;
    if radar_count == 0 {
        let now = Utc::now();
        for (id, uid, name, text, is_new, hours) in [
            ("seed_r1", "seed_tech_lullaby", "UP · TechLullaby", "Posted a calm devlog with a fresh cover — worth a peek tonight.", true, 2),
            ("seed_r2", "seed_systems_garden", "UP · SystemsGarden", "New video: stream caches, latency tuning, no ads.", false, 26),
            ("seed_r3", "seed_mythic_coder", "UP · MythicCoder", "Radar sync: 3 creators added to the watch list.", true, 5),
            ("seed_r4", "seed_calm_canvas", "UP · CalmCanvas", "Mobile grid polish for ACG navigation cards.", false, 48),
        ] {
            sqlx::query(
                "INSERT INTO radar_feed (id, up_uid, creator_name, latest_text, link_url, is_new, published_at) VALUES ($1, $2, $3, $4, 'https://space.bilibili.com/', $5, $6)",
            )
            .bind(id)
            .bind(uid)
            .bind(name)
            .bind(text)
            .bind(is_new)
            .bind(now - chrono::Duration::hours(hours))
            .execute(pool)
            .await?;
        }
        radar_seeded = true;
    }
    Ok((bangumi_seeded, radar_seeded))
}

/// 投稿是否落在现在往前 24 小时内。
pub fn is_fresh(published: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let age = now.signed_duration_since(published);
    age >= chrono::Duration::zero() && age <= chrono::Duration::hours(24)
}

struct LatestPost {
    title: String,
    cover: String,
    published_at: DateTime<Utc>,
    link: String,
}

async fn fetch_latest(state: &AppState, uid: &str) -> Result<LatestPost, FeedError> {
    let (img, sub) = crate::wbi::keys(&state.http)
        .await
        .map_err(FeedError::Upstream)?;
    let query = crate::wbi::signed_query(
        &[("mid", uid), ("pn", "1"), ("ps", "1"), ("order", "pubdate")],
        &img,
        &sub,
        Utc::now().timestamp(),
    );
    let url = format!("https://api.bilibili.com/x/space/wbi/arc/search?{query}");
    let body = bili_json(state, &url, &format!("https://space.bilibili.com/{uid}/")).await?;
    parse_latest(uid, &body)
}

/// 有可用封面时返回同源代理路径，否则返回空字符串。
pub fn proxied_cover(name: &str, remote: &str) -> String {
    if remote_cover(remote).is_some() {
        format!("/api/v1/acg/image/{name}.jpg")
    } else {
        String::new()
    }
}

async fn fetch_bangumi(state: &AppState) -> Result<Vec<BangumiItem>, FeedError> {
    let uid = &state.cfg.bilibili_uid;
    let url = format!("https://api.bilibili.com/x/space/bangumi/follow/list?vmid={uid}&type=1&follow_status=0&pn=1&ps=30");
    let body = bili_json(state, &url, &format!("https://space.bilibili.com/{uid}/")).await?;
    parse_bangumi(&body)
}

async fn bili_json(state: &AppState, url: &str, referer: &str) -> Result<Value, FeedError> {
    let response = state
        .http
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .header("User-Agent", UA)
        .header("Referer", referer)
        .send()
        .await
        .map_err(|error| FeedError::Upstream(error.to_string()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(FeedError::Upstream(format!(
            "B站返回 HTTP {}",
            status.as_u16()
        )));
    }
    response
        .json()
        .await
        .map_err(|error| FeedError::Upstream(error.to_string()))
}

fn parse_latest(uid: &str, body: &Value) -> Result<LatestPost, FeedError> {
    let code = body
        .get("code")
        .and_then(Value::as_i64)
        .ok_or(FeedError::Invalid("投稿响应缺少状态码"))?;
    if code != 0 {
        return Err(FeedError::Upstream(format!("B站返回 {code}")));
    }
    let row = body
        .get("data")
        .and_then(|item| item.get("list"))
        .and_then(|item| item.get("vlist"))
        .and_then(Value::as_array)
        .ok_or(FeedError::Invalid("投稿列表缺失"))?
        .first()
        .ok_or_else(|| FeedError::Upstream("这个 UP 没有投稿".into()))?;
    let title = row
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if title.is_empty() {
        return Err(FeedError::Invalid("投稿标题是空的"));
    }
    let created = row
        .get("created")
        .and_then(Value::as_i64)
        .ok_or(FeedError::Invalid("投稿缺少发布时间"))?;
    let published_at = Utc
        .timestamp_opt(created, 0)
        .single()
        .ok_or(FeedError::Invalid("投稿发布时间无效"))?;
    let bvid = row.get("bvid").and_then(Value::as_str).unwrap_or("");
    let link = if bvid.len() == 12 && bvid.starts_with("BV") {
        format!("https://www.bilibili.com/video/{bvid}")
    } else {
        format!("https://space.bilibili.com/{uid}")
    };
    let cover =
        remote_cover(row.get("pic").and_then(Value::as_str).unwrap_or("")).unwrap_or_default();
    Ok(LatestPost {
        title: title.to_string(),
        cover,
        published_at,
        link,
    })
}

/// 解析追番列表。测试和同步共用。
pub fn parse_bangumi(body: &Value) -> Result<Vec<BangumiItem>, FeedError> {
    let code = body
        .get("code")
        .and_then(Value::as_i64)
        .ok_or(FeedError::Invalid("番剧响应缺少状态码"))?;
    if code != 0 {
        return Err(FeedError::Upstream(format!("B站返回 {code}")));
    }
    let list = body
        .get("data")
        .and_then(|item| item.get("list"))
        .and_then(Value::as_array)
        .ok_or(FeedError::Invalid("番剧列表缺失"))?;
    let mut rows = Vec::with_capacity(list.len());
    for row in list {
        let season = row
            .get("season_id")
            .and_then(Value::as_i64)
            .ok_or(FeedError::Invalid("番剧缺少 season_id"))?;
        if season <= 0 {
            return Err(FeedError::Invalid("番剧 season_id 无效"));
        }
        let title = row
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if title.is_empty() {
            return Err(FeedError::Invalid("番剧标题是空的"));
        }
        let total_count = row.get("total_count").and_then(Value::as_i64).unwrap_or(0);
        let formal = row
            .get("formal_ep_count")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let total = if total_count > 0 { total_count } else { formal };
        let total = i32::try_from(total).map_err(|_| FeedError::Invalid("番剧集数超出范围"))?;
        let new_ep = row.get("new_ep");
        let latest = episode(text(new_ep, "title"), text(new_ep, "index_show"));
        rows.push(BangumiItem {
            id: format!("s{season}"),
            title: title.to_string(),
            watched: first_number(row.get("progress").and_then(Value::as_str).unwrap_or("")),
            total,
            latest_episode: latest,
            cover: remote_cover(row.get("cover").and_then(Value::as_str).unwrap_or(""))
                .unwrap_or_default(),
            link: public_link(row.get("url").and_then(Value::as_str).unwrap_or("")),
        });
    }
    Ok(rows)
}

/// `value` 活得比这次解析久。找不到字段时返回 `'static` 空串，可以缩短成 `'a`。
fn text<'a>(value: Option<&'a Value>, key: &str) -> &'a str {
    value
        .and_then(|item| item.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
}

async fn save_radar(
    pool: &sqlx::PgPool,
    creator: &Creator,
    post: &LatestPost,
    now: DateTime<Utc>,
) -> Result<(), FeedError> {
    let id = format!("r_{}", creator.uid);
    let name = format!("UP · {}", creator.name);
    sqlx::query(
        "INSERT INTO radar_feed (id, up_uid, creator_name, latest_text, cover_path, link_url, is_new, published_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (id) DO UPDATE SET creator_name = EXCLUDED.creator_name, latest_text = EXCLUDED.latest_text, cover_path = EXCLUDED.cover_path, link_url = EXCLUDED.link_url, is_new = EXCLUDED.is_new, published_at = EXCLUDED.published_at, updated_at = now()",
    )
    .bind(&id)
    .bind(&creator.uid)
    .bind(name)
    .bind(&post.title)
    .bind(&post.cover)
    .bind(&post.link)
    .bind(is_fresh(post.published_at, now))
    .bind(post.published_at)
    .execute(pool)
    .await
    .map_err(|error| FeedError::Database(error.to_string()))?;
    Ok(())
}

fn episode(title: &str, index_show: &str) -> i32 {
    for part in [title, index_show] {
        let number = last_number(part);
        if number > 0 {
            return number;
        }
    }
    0
}

fn first_number(value: &str) -> i32 {
    scan_number(value, true)
}

fn last_number(value: &str) -> i32 {
    scan_number(value, false)
}

fn scan_number(value: &str, first: bool) -> i32 {
    let mut digits = String::new();
    let mut found = 0;
    let mut saw = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        if let Some(number) = finish_digits(&mut digits) {
            if first {
                return number;
            }
            found = number;
            saw = true;
        }
    }
    if let Some(number) = finish_digits(&mut digits) {
        return number;
    }
    if saw {
        found
    } else {
        0
    }
}

fn finish_digits(digits: &mut String) -> Option<i32> {
    if digits.is_empty() {
        return None;
    }
    let parsed = digits.parse().ok();
    digits.clear();
    parsed
}

fn public_link(value: &str) -> String {
    let value = value.trim();
    if value.starts_with("https://") || value.starts_with("http://") {
        value.to_string()
    } else {
        String::new()
    }
}

fn valid_uid(uid: &str) -> bool {
    let mut chars = uid.chars();
    matches!(chars.next(), Some('1'..='9'))
        && chars.all(|ch| ch.is_ascii_digit())
        && uid.len() <= 19
        && uid.parse::<i64>().is_ok()
}
