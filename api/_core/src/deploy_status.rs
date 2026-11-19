//! 线上部署状态：读 GitHub Actions 在默认分支上的最新运行结果。
//!
//! 控制台顶栏原来写死「等待发布阶段」，这里改成真实 CI 结论。
//! 结果缓存 60 秒（失败 30 秒防穿透）；GitHub 不可达时返回
//! `configured=true, available=false`，前端显示「状态未知」而不是假状态。

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::AppState;

const CACHE_FOR: Duration = Duration::from_secs(60);
const NEGATIVE_FOR: Duration = Duration::from_secs(30);
const API_BASE: &str = "https://api.github.com";

struct Cached {
    fetched_at: Instant,
    body: Value,
    negative: bool,
}

static CACHE: LazyLock<Mutex<Option<Cached>>> = LazyLock::new(|| Mutex::new(None));

fn cached() -> Option<Value> {
    let guard = CACHE.lock().ok()?;
    let entry = guard.as_ref()?;
    let ttl = if entry.negative {
        NEGATIVE_FOR
    } else {
        CACHE_FOR
    };
    if entry.fetched_at.elapsed() < ttl {
        Some(entry.body.clone())
    } else {
        None
    }
}

fn store(body: Value, negative: bool) {
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(Cached {
            fetched_at: Instant::now(),
            body,
            negative,
        });
    }
}

/// 把 workflow run JSON 映射成控制台需要的最小字段。
pub fn shape_run(run: &Value, workflow: &str) -> Value {
    json!({
        "workflow": workflow,
        "status": run["status"].as_str().unwrap_or(""),
        "conclusion": run["conclusion"].as_str().unwrap_or(""),
        "sha": run["head_sha"].as_str().unwrap_or(""),
        "updatedAt": run["updated_at"].as_str().unwrap_or(""),
        "url": run["html_url"].as_str().unwrap_or(""),
    })
}

/// 未配置 GitHub 凭据/仓库时的诚实兜底。
pub fn unconfigured() -> Value {
    json!({ "configured": false, "available": false, "runs": [] })
}

pub async fn snapshot(state: &AppState) -> Value {
    let (owner, repo, _branch) = match crate::github_publish::repo_from_config(
        &state.cfg.github_token,
        &state.cfg.github_repo,
        &state.cfg.github_branch,
    ) {
        Ok(spec) => spec,
        Err(_) => return unconfigured(),
    };
    if let Some(body) = cached() {
        return body;
    }
    let url = format!("{API_BASE}/repos/{owner}/{repo}/actions/runs?per_page=20");
    let sent = state
        .http
        .get(&url)
        .bearer_auth(&state.cfg.github_token)
        .header("User-Agent", "spume-deploy-status")
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(6))
        .send()
        .await;
    let response = match sent {
        Ok(response) if response.status().is_success() => response,
        Ok(response) => {
            let body = json!({
                "configured": true,
                "available": false,
                "reason": format!("GitHub 返回 {}", response.status().as_u16()),
                "runs": [],
            });
            store(body.clone(), true);
            return body;
        }
        Err(error) => {
            let body = json!({
                "configured": true,
                "available": false,
                "reason": format!("GitHub 请求失败: {error}"),
                "runs": [],
            });
            store(body.clone(), true);
            return body;
        }
    };
    let payload: Value = match response.json().await {
        Ok(payload) => payload,
        Err(error) => {
            let body = json!({
                "configured": true,
                "available": false,
                "reason": format!("GitHub 响应无法解析: {error}"),
                "runs": [],
            });
            store(body.clone(), true);
            return body;
        }
    };
    // 每个工作流只保留最新一次运行；镜像流水线是部署主线，排最前。
    let mut runs: Vec<Value> = Vec::new();
    if let Some(items) = payload["workflow_runs"].as_array() {
        for run in items {
            let name = run["name"].as_str().unwrap_or("");
            if name.is_empty() || runs.iter().any(|item| item["workflow"] == name) {
                continue;
            }
            runs.push(shape_run(run, name));
        }
    }
    runs.sort_by_key(|run| match run["workflow"].as_str().unwrap_or("") {
        name if name.contains("Docker") => 0,
        _ => 1,
    });
    let body = json!({ "configured": true, "available": true, "runs": runs });
    store(body.clone(), false);
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_run_keeps_console_fields() {
        let run = json!({
            "name": "Build and Publish Docker Images",
            "status": "completed",
            "conclusion": "success",
            "head_sha": "abc1234def",
            "updated_at": "2026-11-18T08:30:00Z",
            "html_url": "https://github.com/o/r/actions/runs/1",
        });
        let shaped = shape_run(&run, "Build and Publish Docker Images");
        assert_eq!(shaped["conclusion"], "success");
        assert_eq!(shaped["sha"], "abc1234def");
        assert_eq!(shaped["url"], "https://github.com/o/r/actions/runs/1");
    }

    #[test]
    fn unconfigured_is_honest() {
        let body = unconfigured();
        assert_eq!(body["configured"], false);
        assert_eq!(body["available"], false);
    }
}
