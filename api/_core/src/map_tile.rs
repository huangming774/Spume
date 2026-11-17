//! 「你在哪里」圆框地图的瓦片代理。
//!
//! 天地图 tk 只存在后端环境变量 `TIANDITU_TK` 里，浏览器只请求本站
//! `/api/client/map-tile/{layer}/{z}/{x}/{y}`，由这里异步去天地图拉瓦片
//! 再回传，密钥不再进前端包（原 `VITE_TIANDITU_TK`）。

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::routes::{fail, sniffed_image_type};
use crate::AppState;

/// 允许代理的天地图图层。前端圆框只用矢量底图和矢量注记，
/// 白名单顺带挡掉往查询串里塞 `&` 之类的注入。
const LAYERS: [&str; 2] = ["vec_w", "cva_w"];

/// 天地图 web 墨卡托瓦片支持的级别上限。
const MAX_ZOOM: u32 = 18;

/// 单块瓦片拉取上限。天地图在国内，Vercel 函数在海外，放宽到 8 秒。
const TILE_TIMEOUT: Duration = Duration::from_secs(8);

/// 天地图浏览器端 key 会校验 User-Agent，非浏览器 UA 直接 403
/// 「Key权限类型为:浏览器端，请使用浏览器访问！」，这里伪装成 Chrome。
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// 瓦片上游专用 client。共享 client 的 connect_timeout 只有 1 秒
/// （给国内接口定的），跨洋连天地图握手经常超过 1 秒，这里放宽到 5 秒。
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
/// 图层不在白名单或坐标越界返回 400；没配 tk 返回 503；天地图拉取失败
/// 返回 502（和天气接口一样，前端 `<img>` 的 onError 自己隐藏瓦片）。
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
    let token = state.cfg.tianditu_tk.trim();
    if token.is_empty() {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "MAP_TOKEN_MISSING",
            "地图密钥还没配置",
        );
    }
    let url = tile_url(&state.cfg.tianditu_tile_host, token, layer, z, x, y);
    // 浏览器以前直连天地图时会带本站 Referer，这里同样带上，
    // 与旧版行为一致（key 若绑了域名白名单也能过）。
    let referer = format!("{}/", state.cfg.public_origin.trim_end_matches('/'));
    let response = match TILE_CLIENT
        .get(&url)
        .header("User-Agent", BROWSER_UA)
        .header("Referer", referer)
        .timeout(TILE_TIMEOUT)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, "tianditu tile fetch failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "MAP_TILE_UNAVAILABLE",
                "地图瓦片暂时拿不到",
            );
        }
    };
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "tianditu tile fetch failed");
        return fail(
            StatusCode::BAD_GATEWAY,
            "MAP_TILE_UNAVAILABLE",
            "地图瓦片暂时拿不到",
        );
    }
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(%error, "tianditu tile read failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "MAP_TILE_UNAVAILABLE",
                "地图瓦片暂时拿不到",
            );
        }
    };
    let content_type = sniffed_image_type(&bytes);
    if content_type == HeaderValue::from_static("application/octet-stream") {
        // 天地图偶尔回 200 + 文本错误，不当图片回传。
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

/// 解析并校验瓦片坐标。z 在 1..=18，x/y 都落在 `[0, 2^z)`。
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

/// 只接受十进制数字，防止拼进上游查询串。
fn parse_u32(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 10 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok()
}

/// 拼天地图瓦片地址。host 里的 `{shard}` 换成 `(x + y) % 8`，
/// 和前端旧实现一致，在 t0~t7 八台 DataServer 之间分摊。
/// host 不含占位符时（比如测试指向本地 mock）原样使用。
pub fn tile_url(host: &str, token: &str, layer: &str, z: u32, x: u32, y: u32) -> String {
    let shard = (x + y) % 8;
    let host = host.replace("{shard}", &shard.to_string());
    format!("{host}/DataServer?T={layer}&x={x}&y={y}&l={z}&tk={token}")
}
