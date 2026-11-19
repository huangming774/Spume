use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use taozhiyy_api::music::parse_playlist_id;
use taozhiyy_api::{router, AppState, Config};
use tower::ServiceExt;

#[test]
fn playlist_id_accepts_bare_ids_and_netease_links() {
    assert_eq!(parse_playlist_id("3778678"), Some("3778678".into()));
    assert_eq!(parse_playlist_id("  3778678  "), Some("3778678".into()));
    assert_eq!(
        parse_playlist_id("https://music.163.com/playlist?id=3778678"),
        Some("3778678".into())
    );
    assert_eq!(
        parse_playlist_id("https://music.163.com/#/playlist?id=3778678&foo=999"),
        Some("3778678".into())
    );
    assert_eq!(
        parse_playlist_id("https://music.163.com/#/my/m/music/playlist?id=19723756"),
        Some("19723756".into())
    );
    assert_eq!(
        parse_playlist_id("https://music.163.com/playlist/3778678/"),
        Some("3778678".into())
    );
}

#[test]
fn playlist_id_rejects_junk_and_foreign_input() {
    assert_eq!(parse_playlist_id(""), None);
    assert_eq!(parse_playlist_id("abc"), None);
    assert_eq!(
        parse_playlist_id("https://example.com/playlist?id=123"),
        None
    );
    assert_eq!(parse_playlist_id("123; DROP TABLE users"), None);
    assert_eq!(parse_playlist_id(&"9".repeat(20)), None);
}

async fn request(uri: &str, method: &str, body: &str) -> (StatusCode, serde_json::Value) {
    let app = router(AppState::new(
        Config::for_test(),
        None,
        reqwest::Client::new(),
    ));
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .method(method)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (status, value)
}

#[tokio::test]
async fn music_config_defaults_to_disabled_without_database() {
    let (status, body) = request("/api/client/music", "GET", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], false);
    assert_eq!(body["playlistId"], "");
}

#[tokio::test]
async fn music_playlist_is_404_without_config() {
    let (status, body) = request("/api/client/music/playlist", "GET", "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "MUSIC_NOT_CONFIGURED");
}

#[tokio::test]
async fn owner_music_requires_login() {
    let (status, body) = request("/api/owner/music", "PUT", r#"{"playlist":"3778678"}"#).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "UNAUTHORIZED");
}
