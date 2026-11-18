use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use taozhiyy_api::{router, AppState, Config};
use tower::ServiceExt;

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
async fn deploy_status_is_honest_without_github_config() {
    let (status, body) = request("/api/client/deploy-status", "GET", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["configured"], false);
    assert_eq!(body["available"], false);
    assert_eq!(body["runs"], serde_json::json!([]));
}

#[tokio::test]
async fn friend_approve_requires_login() {
    let (status, body) = request("/api/owner/friends/approve", "POST", r#"{"id":1}"#).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "UNAUTHORIZED");
}

#[tokio::test]
async fn owner_ai_generate_requires_login() {
    let (status, body) = request(
        "/api/owner/ai/generate",
        "POST",
        r#"{"kind":"draft","input":"素材"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "UNAUTHORIZED");
}
