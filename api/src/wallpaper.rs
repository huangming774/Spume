//! 从 Alcy 壁纸接口收集跳转后的 `.webp` 地址，写入图池。
//!
//! 请求不跟随跳转，只用 `Location`。这样同步不会把整张图下载下来。

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::AppState;

const ENDPOINT: &str = "https://t.alcy.cc/pc";
const SOURCE: &str = "alcy";
const REFRESH: i32 = 10;
const ATTEMPTS: i32 = 24;
const MAX_ITEMS: i64 = 120;

/// 一张可以放进图池的壁纸。
pub struct WallpaperCandidate {
    /// 由地址算出的稳定 id。
    pub id: String,
    /// 跳转后的 https `.webp` 地址。
    pub url: String,
}

/// 壁纸同步失败。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WallpaperError {
    /// 跳转地址不能用。
    #[error("{0}")]
    Invalid(&'static str),
    /// 上游没有给出可用图片。
    #[error("{0}")]
    Upstream(String),
    /// 数据库写入失败。
    #[error("{0}")]
    Database(String),
}

/// 把接口地址和 `Location` 解析成一张壁纸。相对路径按接口的源站补全。
pub fn wallpaper_from_redirect(
    endpoint: &str,
    location: &str,
) -> Result<WallpaperCandidate, WallpaperError> {
    let location = location.trim();
    if location.is_empty() {
        return Err(WallpaperError::Invalid("跳转没有地址"));
    }
    let url = if location.starts_with("https://") {
        location.to_string()
    } else if let Some(path) = location.strip_prefix('/') {
        let origin = endpoint.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{origin}/{path}")
    } else {
        return Err(WallpaperError::Invalid(
            "壁纸地址必须是 https 或站内绝对路径",
        ));
    };
    let path = url.split('?').next().unwrap_or(url.as_str());
    if !path.to_ascii_lowercase().ends_with(".webp") {
        return Err(WallpaperError::Invalid("壁纸必须是 webp"));
    }
    Ok(WallpaperCandidate {
        id: wallpaper_id(&url),
        url,
    })
}

/// 停用其它来源的接口壁纸，再向 Alcy 要最多 10 张新图。
pub async fn sync_pool(state: &AppState) -> Result<usize, WallpaperError> {
    let pool = state
        .db
        .as_ref()
        .ok_or(WallpaperError::Database("数据库还没有连上".into()))?;
    sqlx::query("UPDATE wallpapers SET status = 'inactive' WHERE kind = 'api' AND status = 'active' AND source <> $1")
        .bind(SOURCE)
        .execute(pool)
        .await
        .map_err(|error| WallpaperError::Database(error.to_string()))?;
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| WallpaperError::Upstream(error.to_string()))?;
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut last_error = None;
    for _ in 0..ATTEMPTS {
        if items.len() as i32 >= REFRESH {
            break;
        }
        let candidate = match fetch_one(&http).await {
            Ok(candidate) => candidate,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        if !seen.insert(candidate.url.clone()) {
            continue;
        }
        items.push(candidate);
    }
    if items.is_empty() {
        return Err(
            last_error.unwrap_or(WallpaperError::Upstream("壁纸接口没有返回可用图片".into()))
        );
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| WallpaperError::Database(error.to_string()))?;
    for item in &items {
        sqlx::query(
            "INSERT INTO wallpapers (id, url, preview_url, source, author, source_url, license_note, kind, status)
             VALUES ($1, $2, $2, $3, 'Alcy', $2, 'Wallpaper served by the Alcy wallpaper API', 'api', 'active')
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(&item.id)
        .bind(&item.url)
        .bind(SOURCE)
        .execute(&mut *tx)
        .await
        .map_err(|error| WallpaperError::Database(error.to_string()))?;
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wallpapers WHERE kind = 'api' AND status = 'active'",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| WallpaperError::Database(error.to_string()))?;
    let extra = count - MAX_ITEMS;
    if extra > 0 {
        sqlx::query(
            "DELETE FROM wallpapers WHERE id IN (SELECT id FROM wallpapers WHERE kind = 'api' AND status = 'active' ORDER BY added_at ASC, id ASC LIMIT $1)",
        )
        .bind(extra)
        .execute(&mut *tx)
        .await
        .map_err(|error| WallpaperError::Database(error.to_string()))?;
    }
    tx.commit()
        .await
        .map_err(|error| WallpaperError::Database(error.to_string()))?;
    Ok(items.len())
}

async fn fetch_one(http: &reqwest::Client) -> Result<WallpaperCandidate, WallpaperError> {
    let response = http
        .get(ENDPOINT)
        .header(reqwest::header::USER_AGENT, "taozhiyy-wallpaper-pool/1.0")
        .send()
        .await
        .map_err(|error| WallpaperError::Upstream(error.to_string()))?;
    let status = response.status();
    if !status.is_redirection() {
        return Err(WallpaperError::Upstream(format!(
            "壁纸接口返回 HTTP {}",
            status.as_u16()
        )));
    }
    let location = match response.headers().get(reqwest::header::LOCATION) {
        None => return Err(WallpaperError::Invalid("跳转没有地址")),
        Some(value) => value
            .to_str()
            .map_err(|_| WallpaperError::Invalid("跳转地址不是文本"))?,
    };
    wallpaper_from_redirect(ENDPOINT, location)
}

fn wallpaper_id(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("alcy|{url}|{url}"));
    format!("alcy-{}", hex::encode(&hasher.finalize()[..8]))
}
