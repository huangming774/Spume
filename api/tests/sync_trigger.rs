use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use taozhiyy_api::sync_auth::{provided_token, token_matches};
use taozhiyy_api::{router, AppState, Config};
use tower::ServiceExt;

#[test]
fn deployment_token_accepts_the_header_or_bearer_value() {
    assert_eq!(provided_token(Some(" deploy-token "), None), "deploy-token");
    assert_eq!(
        provided_token(None, Some("Bearer deploy-token")),
        "deploy-token"
    );
    assert!(provided_token(None, Some("Basic deploy-token")).is_empty());
    assert!(token_matches("deploy-token", "deploy-token"));
    assert!(!token_matches("deploy-token", "deploy-tokEn"));
    assert!(!token_matches("deploy-token", "short"));
    assert!(!token_matches("", "deploy-token"));
}

#[tokio::test]
async fn anonymous_trigger_asks_for_login() {
    let app = router(AppState::new(
        Config::for_test(),
        None,
        reqwest::Client::new(),
    ));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sync/trigger")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_token_does_not_start_sync() {
    let mut config = Config::for_test();
    config.sync_trigger_token = "deploy-token".into();
    let state = AppState::new(config, None, reqwest::Client::new());
    let app = router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sync/trigger")
                .header("x-sync-trigger-token", "wrong-token")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(!state.sync_running.load(Ordering::SeqCst));
}

#[tokio::test]
async fn running_sync_returns_accepted() {
    let mut config = Config::for_test();
    config.sync_trigger_token = "deploy-token".into();
    let state = AppState::new(config, None, reqwest::Client::new());
    state.sync_running.store(true, Ordering::SeqCst);
    let app = router(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sync/trigger")
                .header("x-sync-trigger-token", "deploy-token")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(payload["queued"], "already_running");
}
