//! 「你在哪里」圆框地图的瓦片代理。
//!
//! 上游是 OpenStreetMap 标准瓦片服务（无需密钥），浏览器只请求本站
//! `/api/client/map-tile/{layer}/{z}/{x}/{y}`，由这里异步去 OSM 拉瓦片
//! 再回传，前端不直连外部服务。

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::routes::{fail, sniffed_image_type};
use crate::AppState;

/// 圆框只用 OSM 标准图层（道路和地名注记在同一张瓦片里）。
/// 白名单顺带挡掉往路径里塞注入内容的请求。
const LAYERS: [&str; 1] = ["osm"];

/// OSM 瓦片支持的级别上限。
const MAX_ZOOM: u32 = 19;

/// 单块瓦片拉取上限。OSM 服务器在海外，放宽到 8 秒。
const TILE_TIMEOUT: Duration = Duration::from_secs(8);

/// OSM 瓦片使用政策要求带能识别应用的 User-Agent，
/// 通用浏览器 UA 或空 UA 可能被限流封禁。
const TILE_UA: &str = "taozhiyy-blog/1.0 (https://www.miku.coffee; where-am-i well map)";

/// 瓦片上游专用 client。共享 client 的 connect_timeout 只有 1 秒
/// （给国内接口定的），跨洋连 OSM 握手经常超过 1 秒，这里放宽到 5 秒。
static TILE_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(8)
        .build()
        .unwrap_or_default()
});

/// `GET /api/client/map-tile/{layer}/{z}/{x}/{y}`。
///
/// 图层不在白名单或坐标越界返回 400；OSM 拉取失败返回 502
/// （和天气接口一样，前端 `<img>` 的 onError 自己隐藏瓦片）。
/// 成功时回传图片字节，带一周的不可变缓存头，浏览器只打一次。
pub async fn tile(
    State(state): State<AppState>,
    Path((layer, z, x, y)): Path<(String, String, String, String)>,
) -> Response {
    let layer = layer.trim();
    if !LAYERS.contains(&layer) {
        return fail(StatusCode::BAD_REQUEST, "INVALID_LAYER", "不支持的地图图层");
    }
    let Some((z, x, y)) = parse_coord(&z, &x, &y) else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_TILE", "瓦片坐标不正确");
    };
    let url = tile_url(&state.cfg.osm_tile_host, z, x, y);
    let response = match TILE_CLIENT
        .get(&url)
        .header("User-Agent", TILE_UA)
        .timeout(TILE_TIMEOUT)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, "osm tile fetch failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "MAP_TILE_UNAVAILABLE",
                "地图瓦片暂时拿不到",
            );
        }
    };
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "osm tile fetch failed");
        return fail(
            StatusCode::BAD_GATEWAY,
            "MAP_TILE_UNAVAILABLE",
            "地图瓦片暂时拿不到",
        );
    }
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(%error, "osm tile read failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "MAP_TILE_UNAVAILABLE",
                "地图瓦片暂时拿不到",
            );
        }
    };
    let content_type = sniffed_image_type(&bytes);
    if content_type == HeaderValue::from_static("application/octet-stream") {
        // 上游偶尔回 200 + 文本错误，不当图片回传。
        return fail(
            StatusCode::BAD_GATEWAY,
            "MAP_TILE_UNAVAILABLE",
            "地图瓦片暂时拿不到",
        );
    }
    (
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=604800, immutable"),
            ),
        ],
        bytes,
    )
        .into_response()
}

/// 解析并校验瓦片坐标。z 在 1..=19，x/y 都落在 `[0, 2^z)`。
pub fn parse_coord(z: &str, x: &str, y: &str) -> Option<(u32, u32, u32)> {
    let z = parse_u32(z)?;
    let x = parse_u32(x)?;
    let y = parse_u32(y)?;
    if z == 0 || z > MAX_ZOOM {
        return None;
    }
    let bound = 1_u32 << z;
    if x >= bound || y >= bound {
        return None;
    }
    Some((z, x, y))
}

/// 只接受十进制数字，防止拼进上游路径。
fn parse_u32(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 10 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok()
}

/// 拼 OSM 瓦片地址，标准 XYZ 方案 `{host}/{z}/{x}/{y}.png`。
/// host 末尾斜杠会被规整；测试指向本地 mock 时同样适用。
pub fn tile_url(host: &str, z: u32, x: u32, y: u32) -> String {
    let host = host.trim_end_matches('/');
    format!("{host}/{z}/{x}/{y}.png")
}
