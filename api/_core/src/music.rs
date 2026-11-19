//! 网易云歌单解析代理。
//!
//! 后台把歌单链接/ID 存进 `site_settings`，浏览器只请求本站
//! `/api/client/music/playlist`，由这里去网易云拉歌单和逐首播放外链。
//! 走的是网页端公开接口（非官方开放平台）：版权/VIP 曲目经常拿不到
//! 外链（url 为 null），统一标记 `playable=false` 让前端跳过；
//! 接口失效时返回 502，前端回退站点自带 loop.mp3。

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::AppState;

/// 网易云网页端接口要求浏览器 UA 和站内 Referer，否则部分接口 403。
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const REFERER: &str = "https://music.163.com/";
/// `os=pc` 让外链按 PC 端下发，可用率更高。
const COOKIE: &str = "os=pc; appver=8.9.70";

const UPSTREAM: &str = "https://music.163.com";
const FETCH_TIMEOUT: Duration = Duration::from_secs(8);
/// 解析结果缓存 10 分钟；失败结果短缓存 30 秒防穿透。
const CACHE_FOR: Duration = Duration::from_secs(600);
const NEGATIVE_FOR: Duration = Duration::from_secs(30);
/// 单个歌单最多解析的曲目数，防止超大歌单拖垮函数。
const MAX_TRACKS: usize = 100;
/// song/detail 与 player/url 每批的曲目数。
const BATCH: usize = 50;

/// 从歌单链接或裸 ID 提取数字 ID。只接受纯数字，防注入上游查询串。
pub fn parse_playlist_id(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 512 {
        return None;
    }
    // 裸 ID
    if raw.bytes().all(|b| b.is_ascii_digit()) && raw.len() <= 16 {
        return Some(raw.to_string());
    }
    // 链接必须真的长得像网易云歌单，避免把随便一串数字当歌单。
    if !raw.contains("163.com") || !(raw.contains("playlist") || raw.contains("id=")) {
        return None;
    }
    // 优先 id=123 查询参数（?id= / &id= / #/playlist?id=）
    if let Some(pos) = raw.find("id=") {
        let digits: String = raw[pos + 3..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() && digits.len() <= 16 {
            return Some(digits);
        }
    }
    // 路径式 /playlist/123
    if let Some(pos) = raw.find("playlist/") {
        let digits: String = raw[pos + 9..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() && digits.len() <= 16 {
            return Some(digits);
        }
    }
    None
}

struct Cached {
    fetched_at: Instant,
    body: Value,
    negative: bool,
}

static CACHE: LazyLock<Mutex<HashMap<String, Cached>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn cached(id: &str) -> Option<(Value, bool)> {
    let guard = CACHE.lock().ok()?;
    let entry = guard.get(id)?;
    let ttl = if entry.negative {
        NEGATIVE_FOR
    } else {
        CACHE_FOR
    };
    if entry.fetched_at.elapsed() < ttl {
        Some((entry.body.clone(), entry.negative))
    } else {
        None
    }
}

fn store(id: &str, body: Value, negative: bool) {
    if let Ok(mut guard) = CACHE.lock() {
        guard.insert(
            id.to_string(),
            Cached {
                fetched_at: Instant::now(),
                body,
                negative,
            },
        );
    }
}

/// 配置改动后清缓存（进程内 + 共享），让前端立刻拿到新歌单。
pub async fn invalidate(state: &AppState, id: &str) {
    if let Ok(mut guard) = CACHE.lock() {
        guard.remove(id);
    }
    crate::cache::del(state, &[&format!("music:pl:{id}")]).await;
}

async fn upstream_get(state: &AppState, url: &str) -> Result<Value, String> {
    let response = state
        .http
        .get(url)
        .header("User-Agent", BROWSER_UA)
        .header("Referer", REFERER)
        .header("Cookie", COOKIE)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response
        .json::<Value>()
        .await
        .map_err(|error| error.to_string())
}

async fn upstream_post_form(state: &AppState, url: &str, body: String) -> Result<Value, String> {
    let response = state
        .http
        .post(url)
        .header("User-Agent", BROWSER_UA)
        .header("Referer", REFERER)
        .header("Cookie", COOKIE)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .timeout(FETCH_TIMEOUT)
        .body(body)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response
        .json::<Value>()
        .await
        .map_err(|error| error.to_string())
}

/// 拉歌单：名称 + 曲目 id 列表（截断到 MAX_TRACKS）。
async fn playlist_meta(state: &AppState, id: &str) -> Result<(String, Vec<String>), String> {
    let value = upstream_get(state, &format!("{UPSTREAM}/api/v6/playlist/detail?id={id}")).await?;
    if value.get("code").and_then(Value::as_i64) != Some(200) {
        return Err("playlist detail rejected".into());
    }
    let playlist = value.get("playlist").ok_or("playlist missing")?;
    let name = playlist
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let ids: Vec<String> = playlist
        .get("trackIds")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_i64))
                .take(MAX_TRACKS)
                .map(|id| id.to_string())
                .collect()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        return Err("playlist empty".into());
    }
    Ok((name, ids))
}

/// 批量曲目详情：id → (歌名, 歌手, 时长毫秒)。
async fn song_details(
    state: &AppState,
    ids: &[String],
) -> Result<HashMap<String, (String, String, i64)>, String> {
    let c = ids
        .iter()
        .map(|id| format!("{{\"id\":{id}}}"))
        .collect::<Vec<_>>()
        .join(",");
    let value = upstream_post_form(
        state,
        &format!("{UPSTREAM}/api/v3/song/detail"),
        format!("c=[{c}]"),
    )
    .await?;
    let mut out = HashMap::new();
    if let Some(songs) = value.get("songs").and_then(Value::as_array) {
        for song in songs {
            let Some(id) = song.get("id").and_then(Value::as_i64) else {
                continue;
            };
            let name = song
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let artist = song
                .get("ar")
                .and_then(Value::as_array)
                .map(|artists| {
                    artists
                        .iter()
                        .filter_map(|a| a.get("name").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(" / ")
                })
                .unwrap_or_default();
            let duration = song.get("dt").and_then(Value::as_i64).unwrap_or(0);
            out.insert(id.to_string(), (name, artist, duration));
        }
    }
    Ok(out)
}

/// 批量播放外链：id → 可播 url（拿不到为 None）。
async fn player_urls(
    state: &AppState,
    ids: &[String],
) -> Result<HashMap<String, Option<String>>, String> {
    let ids_json = format!("[{}]", ids.join(","));
    let url = format!(
        "{UPSTREAM}/api/song/enhance/player/url?ids={}&br=320000",
        urlencoding_min(&ids_json)
    );
    let value = upstream_get(state, &url).await?;
    let mut out = HashMap::new();
    if let Some(items) = value.get("data").and_then(Value::as_array) {
        for item in items {
            let Some(id) = item.get("id").and_then(Value::as_i64) else {
                continue;
            };
            let playable = item
                .get("url")
                .and_then(Value::as_str)
                .filter(|u| u.starts_with("http"))
                .map(str::to_string);
            out.insert(id.to_string(), playable);
        }
    }
    Ok(out)
}

/// 只需要 encode 方括号和逗号，避免为此引一个 crate。
fn urlencoding_min(raw: &str) -> String {
    raw.replace('[', "%5B")
        .replace(']', "%5D")
        .replace(',', "%2C")
}

/// 解析整个歌单为前端播放列表。失败返回 Err（路由层转 502）。
/// 缓存层级：进程内 → 共享 Redis → 上游；未配 Redis 时与原行为一致。
pub async fn resolve(state: &AppState, id: &str) -> Result<Value, String> {
    if let Some((body, negative)) = cached(id) {
        return if negative {
            Err("cached failure".into())
        } else {
            Ok(body)
        };
    }
    let key = format!("music:pl:{id}");
    if let Some((body, negative)) = crate::cache::get(state, &key).await {
        store(id, body.clone(), negative);
        return if negative {
            Err("cached failure".into())
        } else {
            Ok(body)
        };
    }
    let result = resolve_uncached(state, id).await;
    match &result {
        Ok(body) => {
            store(id, body.clone(), false);
            crate::cache::set(state, &key, body, false, CACHE_FOR).await;
        }
        Err(_) => {
            store(id, json!({}), true);
            crate::cache::set(state, &key, &json!({}), true, NEGATIVE_FOR).await;
        }
    }
    result
}

async fn resolve_uncached(state: &AppState, id: &str) -> Result<Value, String> {
    let (name, ids) = playlist_meta(state, id).await?;
    let mut tracks = Vec::new();
    for chunk in ids.chunks(BATCH) {
        let details = song_details(state, chunk).await.unwrap_or_default();
        let urls = player_urls(state, chunk).await.unwrap_or_default();
        for song_id in chunk {
            let (title, artist, duration) = details
                .get(song_id)
                .cloned()
                .unwrap_or_else(|| (String::new(), String::new(), 0));
            let url = urls.get(song_id).cloned().flatten();
            tracks.push(json!({
                "id": song_id,
                "name": title,
                "artist": artist,
                "duration": duration,
                "url": url.clone().unwrap_or_default(),
                "playable": url.is_some(),
            }));
        }
    }
    Ok(json!({
        "playlistId": id,
        "playlistName": name,
        "tracks": tracks,
    }))
}
