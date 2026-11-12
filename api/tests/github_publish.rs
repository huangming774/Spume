use std::sync::{Arc, Mutex};

use base64::Engine;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use taozhiyy_api::github_publish::{publish_friend, RepoSpec, FRIEND_CARDS_PATH};
use taozhiyy_api::js_source::FriendInput;

#[derive(Clone)]
struct Saved {
    content: Arc<Mutex<String>>,
}

async fn file(State(saved): State<Saved>) -> Json<Value> {
    let content = saved.content.lock().expect("saved content").clone();
    Json(
        json!({"sha": "file-sha", "encoding": "base64", "content": base64::engine::general_purpose::STANDARD.encode(content)}),
    )
}

async fn put_file(State(saved): State<Saved>, request: Request) -> Result<Json<Value>, StatusCode> {
    let bytes = axum::body::to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let payload: Value = serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    let encoded = payload
        .get("content")
        .and_then(Value::as_str)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let text = String::from_utf8(decoded).map_err(|_| StatusCode::BAD_REQUEST)?;
    *saved.content.lock().expect("saved content") = text;
    Ok(Json(
        json!({"content": {"path": FRIEND_CARDS_PATH}, "commit": {"sha": "commit-sha"}}),
    ))
}

async fn branch_head() -> Json<Value> {
    Json(json!({"object": {"sha": "base-sha"}}))
}

async fn create_ref() -> Json<Value> {
    Json(json!({"ref": "refs/heads/owner/friend"}))
}

async fn create_pr() -> Json<Value> {
    Json(json!({"number": 7, "html_url": "https://github.com/called/rustblog/pull/7"}))
}

#[tokio::test]
async fn friend_publish_writes_the_card_and_opens_a_pull_request() {
    let saved = Saved {
        content: Arc::new(Mutex::new("export const friendCards = [\n];\n".to_string())),
    };
    let app = Router::new()
        .route(
            "/repos/{owner}/{repo}/contents/{*path}",
            get(file).put(put_file),
        )
        .route(
            "/repos/{owner}/{repo}/git/ref/heads/{*branch}",
            get(branch_head),
        )
        .route("/repos/{owner}/{repo}/git/refs", post(create_ref))
        .route("/repos/{owner}/{repo}/pulls", post(create_pr))
        .with_state(saved.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("github mock");
    });

    let http = reqwest::Client::new();
    let base = format!("http://{address}");
    let spec = RepoSpec {
        api_base: &base,
        token: "test-token",
        owner: "called",
        repo: "rustblog",
        branch: "master",
    };
    let now = Utc.with_ymd_and_hms(2026, 11, 12, 1, 2, 3).unwrap();
    let published = publish_friend(
        &http,
        &spec,
        FriendInput {
            name: "桃之夭夭",
            desc: "个人站",
            url: "https://taozhiyy.top",
            avatar: "https://images.example/avatar.jpg",
        },
        now,
    )
    .await
    .expect("publish");

    assert!(published.changed);
    assert_eq!(published.commit_sha, "commit-sha");
    assert_eq!(published.pull_request_number, 7);
    assert!(published
        .branch
        .starts_with("owner/friend-20261112-010203-"));
    let stored = saved.content.lock().expect("saved content").clone();
    assert!(stored.contains("name: \"桃之夭夭\""));
    assert!(stored.contains("note: \"FRIEND\""));
}
