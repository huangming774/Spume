use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use taozhiyy_api::github_publish::RepoSpec;
use taozhiyy_api::repo_commits::fetch_commits;

async fn commits(Query(params): Query<Vec<(String, String)>>) -> Json<Value> {
    // 校验调用方带上了分支与条数参数，模拟 GitHub 的查询契约。
    let sha = params
        .iter()
        .find(|(key, _)| key == "sha")
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    assert_eq!(sha, "main");
    Json(json!([
        {
            "sha": "08bbc896f668de4d9eb0e2818e5f14b10cd4cb60",
            "html_url": "https://github.com/o/r/commit/08bbc89",
            "commit": {
                "message": "feat: 留言客户端信息图标行\n\n解析 UA 存系统/浏览器标签",
                "author": { "name": "栀子sun", "date": "2026-11-20T08:30:00Z" },
                "committer": { "name": "栀子sun", "date": "2026-11-20T08:30:00Z" }
            },
            "author": { "login": "huangming774" }
        },
        {
            "sha": "b28a10e317e24aa8634622ddce778aae1f42b195",
            "html_url": "https://github.com/o/r/commit/b28a10e",
            "commit": {
                "message": "chore: 全面清理旧外链与旧账号引用",
                "author": { "name": "栀子sun", "date": "2026-11-19T10:00:00Z" },
                "committer": { "name": "栀子sun", "date": "2026-11-19T10:00:00Z" }
            },
            "author": { "login": "huangming774" }
        },
        { "sha": "", "commit": { "message": "残缺条目应被丢弃" } }
    ]))
}

async fn broken() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn spawn(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("github mock");
    });
    format!("http://{address}")
}

#[tokio::test]
async fn fetch_commits_shapes_the_timeline_fields() {
    let base = spawn(Router::new().route("/repos/{owner}/{repo}/commits", get(commits))).await;
    let spec = RepoSpec {
        api_base: &base,
        token: "test-token",
        owner: "o",
        repo: "r",
        branch: "main",
    };
    let shaped = fetch_commits(&reqwest::Client::new(), &spec)
        .await
        .expect("fetch");
    assert_eq!(shaped.len(), 2);
    assert_eq!(shaped[0]["shortSha"], "08bbc89");
    assert_eq!(shaped[0]["title"], "feat: 留言客户端信息图标行");
    assert_eq!(shaped[0]["body"], "解析 UA 存系统/浏览器标签");
    assert_eq!(shaped[0]["kind"], "feat");
    assert_eq!(shaped[0]["date"], "2026-11-20T08:30:00Z");
    assert_eq!(shaped[1]["kind"], "chore");
    assert_eq!(shaped[1]["body"], "");
    // 响应里不允许出现 token。
    let serialized = serde_json::to_string(&shaped).expect("serialize");
    assert!(!serialized.contains("test-token"));
}

#[tokio::test]
async fn fetch_commits_reports_http_failure() {
    let base = spawn(Router::new().route("/repos/{owner}/{repo}/commits", get(broken))).await;
    let spec = RepoSpec {
        api_base: &base,
        token: "test-token",
        owner: "o",
        repo: "r",
        branch: "main",
    };
    let error = fetch_commits(&reqwest::Client::new(), &spec)
        .await
        .expect_err("should fail");
    assert!(error.contains("500"), "unexpected error: {error}");
}
