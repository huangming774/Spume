//! 仓库提交纪事：读 GitHub 配置分支上的提交说明，供「创造纪事」时间轴展示。
//!
//! 结果缓存 10 分钟（失败 30 秒防穿透），进程内缓存 + Upstash 共享缓存双层，
//! 与 [crate::deploy_status] 同一套模式。GitHub 不可达时返回
//! `configured=true, available=false`，前端显示「暂时无法同步」而不是假记录。
//! token 只进 Authorization 头，绝不出现在响应里。

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::github_publish::RepoSpec;
use crate::AppState;

const CACHE_FOR: Duration = Duration::from_secs(600);
const NEGATIVE_FOR: Duration = Duration::from_secs(30);
const API_BASE: &str = "https://api.github.com";
/// 拉取条数：时间轴只展示最近一段生长过程，不做全量归档。
const PER_PAGE: u8 = 30;

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

/// 提交说明第一行作为标题，其余作为详情正文。
pub fn split_message(message: &str) -> (String, String) {
    let trimmed = message.trim();
    match trimmed.split_once('\n') {
        Some((title, rest)) => (title.trim().to_string(), rest.trim().to_string()),
        None => (trimmed.to_string(), String::new()),
    }
}

/// 从 `feat:` / `fix(scope):` 这类约定式前缀提取类型；没有前缀返回空串。
pub fn commit_kind(title: &str) -> &'static str {
    let head = title.split(':').next().unwrap_or("");
    let kind = head.split('(').next().unwrap_or("").trim();
    match kind {
        "feat" => "feat",
        "fix" => "fix",
        "perf" => "perf",
        "refactor" => "refactor",
        "style" => "style",
        "docs" => "docs",
        "test" => "test",
        "build" => "build",
        "ci" => "ci",
        "chore" => "chore",
        "revert" => "revert",
        _ => "",
    }
}

/// 把 GitHub commits API 的单条 JSON 映射成时间轴需要的最小字段。
pub fn shape_commit(item: &Value) -> Value {
    let sha = item["sha"].as_str().unwrap_or("");
    let message = item["commit"]["message"].as_str().unwrap_or("");
    let (title, body) = split_message(message);
    // 优先作者时间；合并提交等没有 author 时退回 committer。
    let date = item["commit"]["author"]["date"]
        .as_str()
        .filter(|value| !value.is_empty())
        .or_else(|| item["commit"]["committer"]["date"].as_str())
        .unwrap_or("");
    let author = item["commit"]["author"]["name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .or_else(|| item["author"]["login"].as_str())
        .unwrap_or("");
    json!({
        "sha": sha,
        "shortSha": sha.chars().take(7).collect::<String>(),
        "title": title,
        "body": body,
        "kind": commit_kind(&title),
        "date": date,
        "author": author,
        "url": item["html_url"].as_str().unwrap_or(""),
    })
}

/// 分支名会拼进 GitHub URL，只放行 Git 常规字符，规则与发布模块一致。
fn safe_branch(branch: &str) -> bool {
    !branch.is_empty()
        && !branch.contains("..")
        && !branch.starts_with('/')
        && !branch.ends_with('/')
        && branch
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '/' | '-'))
}

/// 未配置 GitHub 凭据/仓库时的诚实兜底。
pub fn unconfigured() -> Value {
    json!({ "configured": false, "available": false, "commits": [] })
}

fn unavailable(reason: String) -> Value {
    json!({ "configured": true, "available": false, "reason": reason, "commits": [] })
}

/// 拉取并裁剪提交列表。独立函数便于测试注入本地 mock 地址。
pub async fn fetch_commits(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
) -> Result<Vec<Value>, String> {
    let url = format!(
        "{}/repos/{}/{}/commits?sha={}&per_page={PER_PAGE}",
        repo.api_base, repo.owner, repo.repo, repo.branch
    );
    let mut request = http
        .get(&url)
        .header("User-Agent", "spume-repo-commits")
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(6));
    // 公开仓库允许匿名读取；只有配置了非空 token 才带凭据，避免空 Bearer 被 401。
    if !repo.token.is_empty() {
        request = request.bearer_auth(repo.token);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("GitHub 请求失败: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("GitHub 返回 {}", response.status().as_u16()));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("GitHub 响应无法解析: {error}"))?;
    let items = payload
        .as_array()
        .ok_or_else(|| "GitHub 响应无法解析: 预期提交数组".to_string())?;
    Ok(items
        .iter()
        .map(shape_commit)
        .filter(|commit| !commit["sha"].as_str().unwrap_or("").is_empty())
        .collect())
}

pub async fn snapshot(state: &AppState) -> Value {
    // 只读公开仓库：有 token 用 token（限额高），没有就匿名读，仓库未配置才兜底。
    let Some((owner, repo)) = state.cfg.github_repo.split_once('/') else {
        return unconfigured();
    };
    let branch = state.cfg.github_branch.as_str();
    if owner.is_empty() || repo.is_empty() || repo.contains('/') || !safe_branch(branch) {
        return unconfigured();
    }
    if let Some(body) = cached() {
        return body;
    }
    let cache_key = format!(
        "repo:commits:{}:{}",
        crate::cache::sanitize(&format!("{owner}/{repo}")),
        crate::cache::sanitize(branch)
    );
    if let Some((body, negative)) = crate::cache::get(state, &cache_key).await {
        store(body.clone(), negative);
        return body;
    }
    let spec = RepoSpec {
        api_base: API_BASE,
        token: &state.cfg.github_token,
        owner,
        repo,
        branch,
    };
    let body = match fetch_commits(&state.http, &spec).await {
        Ok(commits) => {
            let body = json!({
                "configured": true,
                "available": true,
                "repo": format!("{owner}/{repo}"),
                "branch": branch,
                "commits": commits,
            });
            store(body.clone(), false);
            crate::cache::set(state, &cache_key, &body, false, CACHE_FOR).await;
            body
        }
        Err(reason) => {
            let body = unavailable(reason);
            store(body.clone(), true);
            crate::cache::set(state, &cache_key, &body, true, NEGATIVE_FOR).await;
            body
        }
    };
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_message_separates_title_and_body() {
        let (title, body) = split_message("feat: 新增提交时间轴\n\n- 后端接口\n- 前端展示\n");
        assert_eq!(title, "feat: 新增提交时间轴");
        assert_eq!(body, "- 后端接口\n- 前端展示");
        let (title, body) = split_message("  chore: 单行说明  ");
        assert_eq!(title, "chore: 单行说明");
        assert_eq!(body, "");
    }

    #[test]
    fn commit_kind_reads_conventional_prefix() {
        assert_eq!(commit_kind("feat: 新功能"), "feat");
        assert_eq!(commit_kind("fix(api): 修边界"), "fix");
        assert_eq!(commit_kind("perf: 降粒子"), "perf");
        assert_eq!(commit_kind("把站点正文改存数据库"), "");
        assert_eq!(commit_kind("Update README.md"), "");
    }

    #[test]
    fn shape_commit_keeps_timeline_fields() {
        let item = json!({
            "sha": "08bbc896f668de4d9eb0e2818e5f14b10cd4cb60",
            "html_url": "https://github.com/o/r/commit/08bbc89",
            "commit": {
                "message": "feat: 留言客户端信息\n\n解析 UA 存标签",
                "author": { "name": "栀子sun", "date": "2026-11-20T08:30:00Z" },
                "committer": { "name": "GitHub", "date": "2026-11-20T09:00:00Z" }
            },
            "author": { "login": "huangming774" }
        });
        let shaped = shape_commit(&item);
        assert_eq!(shaped["shortSha"], "08bbc89");
        assert_eq!(shaped["title"], "feat: 留言客户端信息");
        assert_eq!(shaped["body"], "解析 UA 存标签");
        assert_eq!(shaped["kind"], "feat");
        assert_eq!(shaped["date"], "2026-11-20T08:30:00Z");
        assert_eq!(shaped["author"], "栀子sun");
        assert_eq!(shaped["url"], "https://github.com/o/r/commit/08bbc89");
    }

    #[test]
    fn shape_commit_falls_back_to_committer_date() {
        let item = json!({
            "sha": "abc1234def5678",
            "commit": {
                "message": "chore: merge",
                "author": null,
                "committer": { "name": "GitHub", "date": "2026-11-19T00:00:00Z" }
            },
            "author": { "login": "web-flow" }
        });
        let shaped = shape_commit(&item);
        assert_eq!(shaped["date"], "2026-11-19T00:00:00Z");
        assert_eq!(shaped["author"], "web-flow");
    }

    #[test]
    fn unconfigured_is_honest() {
        let body = unconfigured();
        assert_eq!(body["configured"], false);
        assert_eq!(body["available"], false);
        assert_eq!(body["commits"].as_array().map(Vec::len), Some(0));
    }
}
