use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use taozhiyy_api::map_tile::{parse_coord, tile_url};
use taozhiyy_api::{router, AppState, Config};
use tower::ServiceExt;

#[test]
fn tile_url_moves_the_shard_placeholder_like_the_old_frontend() {
    assert_eq!(
        tile_url(
            "https://t{shard}.tianditu.gov.cn",
            "tk",
            "vec_w",
            12,
            2047,
            1365
        ),
        "https://t4.tianditu.gov.cn/DataServer?T=vec_w&x=2047&y=1365&l=12&tk=tk"
    );
    // 测试或自建网关不含占位符时原样使用。
    assert_eq!(
        tile_url("http://127.0.0.1:9", "tk", "cva_w", 12, 0, 0),
        "http://127.0.0.1:9/DataServer?T=cva_w&x=0&y=0&l=12&tk=tk"
    );
}

#[test]
fn tile_coords_are_bounded_by_the_zoom_level() {
    assert_eq!(parse_coord("12", "0", "0"), Some((12, 0, 0)));
    assert_eq!(parse_coord("12", "4095", "4095"), Some((12, 4095, 4095)));
    // x/y 超出 [0, 2^z)。
    assert_eq!(parse_coord("12", "4096", "0"), None);
    assert_eq!(parse_coord("1", "0", "2"), None);
    // z 为 0 或超过 18 级。
    assert_eq!(parse_coord("0", "0", "0"), None);
    assert_eq!(parse_coord("19", "0", "0"), None);
    // 非十进制、空串、超长数字都拒绝。
    assert_eq!(parse_coord("12", "0x10", "0"), None);
    assert_eq!(parse_coord("12", "", "0"), None);
    assert_eq!(parse_coord("12", "00000000000", "0"), None);
}

async fn tile_response(uri: &str, tianditu_tk: &str) -> (StatusCode, Vec<u8>) {
    let mut config = Config::for_test();
    config.tianditu_tk = tianditu_tk.into();
    // 指到不存在的端口：瓦片拉取失败时也要走 502 分支，而不是连不上就超时。
    config.tianditu_tile_host = "http://127.0.0.1:9".into();
    let app = router(AppState::new(config, None, reqwest::Client::new()));
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (status, bytes.to_vec())
}

#[tokio::test]
async fn tile_rejects_unknown_layer() {
    let (status, body) = tile_response("/api/client/map-tile/img_w/12/1/1", "tk").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["error"], "INVALID_LAYER");
}

#[tokio::test]
async fn tile_rejects_out_of_range_coords() {
    let (status, body) = tile_response("/api/client/map-tile/vec_w/12/4096/0", "tk").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["error"], "INVALID_TILE");
}

#[tokio::test]
async fn tile_without_token_reports_missing_key() {
    let (status, body) = tile_response("/api/client/map-tile/vec_w/12/1/1", "").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["error"], "MAP_TOKEN_MISSING");
}

#[tokio::test]
async fn tile_upstream_failure_is_bad_gateway() {
    let (status, body) = tile_response("/api/client/map-tile/vec_w/12/1/1", "tk").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(payload["error"], "MAP_TILE_UNAVAILABLE");
}

#[tokio::test]
async fn tile_proxies_image_bytes_with_cache_headers() {
    use axum::extract::{Query, State};
    use axum::routing::get;
    use axum::Router;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // 本地 mock 天地图：记录收到的查询串，回一张最小 PNG。
    #[derive(Clone)]
    struct Seen(Arc<Mutex<HashMap<String, String>>>);

    async fn mock_tile(
        State(seen): State<Seen>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Vec<u8> {
        *seen.0.lock().expect("seen") = params;
        vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']
    }

    let seen = Seen(Arc::new(Mutex::new(HashMap::new())));
    let mock = Router::new()
        .route("/DataServer", get(mock_tile))
        .with_state(seen.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        axum::serve(listener, mock).await.expect("tile mock");
    });

    let mut config = Config::for_test();
    config.tianditu_tk = "secret-tk".into();
    config.tianditu_tile_host = format!("http://{address}");
    let app = router(AppState::new(config, None, reqwest::Client::new()));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/client/map-tile/vec_w/12/2047/1365")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/png")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=604800, immutable")
    );
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");

    // 后端把 tk 拼进上游 URL，回给浏览器的只有图片字节。
    let params = seen.0.lock().expect("seen").clone();
    assert_eq!(params.get("T").map(String::as_str), Some("vec_w"));
    assert_eq!(params.get("x").map(String::as_str), Some("2047"));
    assert_eq!(params.get("y").map(String::as_str), Some("1365"));
    assert_eq!(params.get("l").map(String::as_str), Some("12"));
    assert_eq!(params.get("tk").map(String::as_str), Some("secret-tk"));
}
