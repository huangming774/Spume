//! 把友链、碎语和相册改动写回 GitHub。
//!
//! `RepoSpec` 里的字符串都借用配置，只在这一次请求里使用。
//! 友链走新分支和 Pull Request；碎语和相册直接写配置里的分支。

use base64::Engine;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use thiserror::Error;

use crate::js_source::{self, FriendInput, GalleryInput, MomentInput, SourceError};
use crate::post_markdown::conflict_path;

/// `main/src/data/friendCards.js`
pub const FRIEND_CARDS_PATH: &str = "main/src/data/friendCards.js";
/// `main/src/data/moments.js`
pub const MOMENTS_PATH: &str = "main/src/data/moments.js";
/// `main/src/data/galleryAlbums.js`
pub const GALLERY_PATH: &str = "main/src/data/galleryAlbums.js";

/// 一次 GitHub 写回使用的仓库。字段借用调用方，不能跨请求保存。
pub struct RepoSpec<'a> {
    /// `https://api.github.com`，测试时换成本地地址。
    pub api_base: &'a str,
    /// Personal access token。只放进 Authorization 头。
    pub token: &'a str,
    /// 仓库所有者。
    pub owner: &'a str,
    /// 仓库名，不含所有者。
    pub repo: &'a str,
    /// 文章和碎语写入的基线分支，友链 PR 也以它为 base。
    pub branch: &'a str,
}

/// 友链发布结果。
pub struct FriendPublished {
    /// 站点名称。
    pub name: String,
    /// 站点地址。
    pub url: String,
    /// 仓库内路径。
    pub path: String,
    /// 新提交。未改动时为空。
    pub commit_sha: String,
    /// 是否新建了提交。
    pub changed: bool,
    /// 实际写入的分支。未改动时是基线分支。
    pub branch: String,
    /// 提交说明。
    pub commit_message: String,
    /// Pull Request 页面。未改动时为空。
    pub pull_request_url: String,
    /// Pull Request 编号。未改动时为 0。
    pub pull_request_number: i64,
}

/// 碎语发布结果。
pub struct MomentPublished {
    /// 年份。
    pub year: String,
    /// 月.日。
    pub date: String,
    /// 分类。
    pub kind: String,
    /// 仓库内路径。
    pub path: String,
    /// 新提交。未改动时为空。
    pub commit_sha: String,
    /// 是否新建了提交。
    pub changed: bool,
    /// 写入的分支。
    pub branch: String,
    /// 提交说明。
    pub commit_message: String,
}

/// 相册发布结果。
pub struct GalleryPublished {
    /// 相册 id。
    pub album_id: String,
    /// 图片地址。
    pub image_url: String,
    /// 仓库内路径。
    pub path: String,
    /// 新提交。未改动时为空。
    pub commit_sha: String,
    /// 是否新建了提交。
    pub changed: bool,
    /// 写入的分支。
    pub branch: String,
    /// 提交说明。
    pub commit_message: String,
}

/// 发布失败。GitHub 的 HTTP 状态保留下来，方便接口映射 502。
#[derive(Debug, Error)]
pub enum PublishError {
    /// Token、仓库或分支没有配全。
    #[error("站长发布尚未在服务器配置。")]
    NotConfigured,
    /// 草稿或数据文件不能改。
    #[error("{0}")]
    Invalid(&'static str),
    /// GitHub 返回了 4xx 或 5xx。
    #[error("GitHub 返回 {status}: {message}")]
    GitHub {
        /// HTTP 状态码。
        status: u16,
        /// GitHub `message`，过长时截断。
        message: String,
    },
    /// 连接 GitHub 失败。
    #[error("GitHub 请求失败: {0}")]
    Transport(String),
    /// 成功状态的响应不是预期 JSON。
    #[error("GitHub 响应无法解析: {0}")]
    Response(String),
}

impl From<SourceError> for PublishError {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::Invalid(message) => Self::Invalid(message),
        }
    }
}

/// 从 `owner/repo` 拆出仓库。分支为空或包含 `..` 时视为未配置。
pub fn repo_from_config<'a>(
    token: &'a str,
    repo: &'a str,
    branch: &'a str,
) -> Result<(&'a str, &'a str, &'a str), PublishError> {
    let (owner, name) = repo.split_once('/').ok_or(PublishError::NotConfigured)?;
    if token.is_empty()
        || owner.is_empty()
        || name.is_empty()
        || name.contains('/')
        || !safe_branch(branch)
    {
        return Err(PublishError::NotConfigured);
    }
    Ok((owner, name, branch))
}

/// 发布一条友链。已存在的 URL 不会创建分支。
pub async fn publish_friend(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    input: FriendInput<'_>,
    now: DateTime<Utc>,
) -> Result<FriendPublished, PublishError> {
    let file = get_file(http, repo, FRIEND_CARDS_PATH).await?;
    let update = js_source::update_friend_cards(&file.content, input)?;
    let commit_message = format!(
        "feat: publish friend link {} {}",
        update.name,
        now.format("%Y%m%d-%H%M%S")
    );
    if !update.changed {
        return Ok(unchanged_friend(
            update.name,
            update.url,
            repo.branch,
            &commit_message,
        ));
    }
    let branch = friend_branch(update.name, now);
    let head = branch_sha(http, repo, repo.branch).await?;
    create_branch(http, repo, &branch, &head).await?;
    let put = put_file(
        http,
        repo,
        FRIEND_CARDS_PATH,
        &commit_message,
        &update.source,
        &file.sha,
        &branch,
    )
    .await?;
    let pull = create_pull_request(
        http,
        repo,
        &format!("Publish friend link: {}", update.name),
        &format!(
            "Created from the owner console friend link publish flow.\n\nFriend: {}\nURL: {}",
            update.name, update.url
        ),
        &branch,
    )
    .await?;
    Ok(FriendPublished {
        name: update.name.to_string(),
        url: update.url.to_string(),
        path: put.path,
        commit_sha: put.sha,
        changed: true,
        branch,
        commit_message,
        pull_request_url: pull.url,
        pull_request_number: pull.number,
    })
}

/// 发布一条碎语，直接写基线分支。
pub async fn publish_moment(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    input: MomentInput<'_>,
    now: DateTime<Utc>,
) -> Result<MomentPublished, PublishError> {
    let file = get_file(http, repo, MOMENTS_PATH).await?;
    let update = js_source::update_moments(&file.content, input, now)?;
    let commit_message = format!(
        "feat: publish moment {}-{} {}",
        update.year,
        update.date,
        now.format("%Y%m%d-%H%M%S")
    );
    let put = put_file(
        http,
        repo,
        MOMENTS_PATH,
        &commit_message,
        &update.source,
        &file.sha,
        repo.branch,
    )
    .await?;
    Ok(MomentPublished {
        year: update.year,
        date: update.date,
        kind: update.kind,
        path: put.path,
        commit_sha: put.sha,
        changed: update.changed,
        branch: repo.branch.to_string(),
        commit_message,
    })
}

/// 发布一张相册图片，直接写基线分支。重复图片不产生提交。
pub async fn publish_gallery(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    input: GalleryInput<'_>,
    now: DateTime<Utc>,
) -> Result<GalleryPublished, PublishError> {
    let file = get_file(http, repo, GALLERY_PATH).await?;
    let update = js_source::update_gallery(&file.content, input)?;
    let commit_message = format!(
        "feat: publish gallery image to {} {}",
        update.album_id,
        now.format("%Y%m%d-%H%M%S")
    );
    if !update.changed {
        return Ok(GalleryPublished {
            album_id: update.album_id,
            image_url: update.image_url,
            path: GALLERY_PATH.to_string(),
            commit_sha: String::new(),
            changed: false,
            branch: repo.branch.to_string(),
            commit_message,
        });
    }
    let put = put_file(
        http,
        repo,
        GALLERY_PATH,
        &commit_message,
        &update.source,
        &file.sha,
        repo.branch,
    )
    .await?;
    Ok(GalleryPublished {
        album_id: update.album_id,
        image_url: update.image_url,
        path: put.path,
        commit_sha: put.sha,
        changed: true,
        branch: repo.branch.to_string(),
        commit_message,
    })
}

/// 发布后的文章。
pub struct ArticlePublished {
    /// 仓库内路径。冲突时会带时间后缀。
    pub path: String,
    /// 新提交。
    pub commit_sha: String,
    /// 写入的分支。
    pub branch: String,
    /// 提交说明。
    pub commit_message: String,
}

/// 新建一篇文章。目标路径已存在时改用带时间的路径再写一次。
pub async fn publish_article(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    title: &str,
    markdown: &str,
    path: &str,
    now: DateTime<Utc>,
) -> Result<ArticlePublished, PublishError> {
    let message = format!("feat: publish {title}");
    let created = match create_file(http, repo, path, &message, markdown).await {
        Ok(put) => put,
        Err(error) if is_conflict(&error) => {
            create_file(http, repo, &conflict_path(path, now), &message, markdown).await?
        }
        Err(error) => return Err(error),
    };
    Ok(ArticlePublished {
        path: created.path,
        commit_sha: created.sha,
        branch: repo.branch.to_string(),
        commit_message: message,
    })
}

fn is_conflict(error: &PublishError) -> bool {
    matches!(
        error,
        PublishError::GitHub {
            status: 409 | 422,
            ..
        }
    )
}

fn unchanged_friend(name: &str, url: &str, branch: &str, commit_message: &str) -> FriendPublished {
    FriendPublished {
        name: name.to_string(),
        url: url.to_string(),
        path: FRIEND_CARDS_PATH.to_string(),
        commit_sha: String::new(),
        changed: false,
        branch: branch.to_string(),
        commit_message: commit_message.to_string(),
        pull_request_url: String::new(),
        pull_request_number: 0,
    }
}

fn friend_branch(name: &str, now: DateTime<Utc>) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in name.trim().chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        if slug.len() >= 40 {
            break;
        }
    }
    let slug = slug.trim_matches('-');
    let slug = if slug.is_empty() { "link" } else { slug };
    format!(
        "owner/friend-{}-{:06}-{slug}",
        now.format("%Y%m%d-%H%M%S"),
        now.timestamp_subsec_micros()
    )
}

struct RemoteFile {
    sha: String,
    content: String,
}

struct PutResult {
    path: String,
    sha: String,
}

struct PullResult {
    number: i64,
    url: String,
}

async fn get_file(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    path: &str,
) -> Result<RemoteFile, PublishError> {
    let url = format!(
        "{}/repos/{}/{}/contents/{}?ref={}",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo),
        escape_path(path),
        path_escape(repo.branch)
    );
    let value = send(http, repo.token, reqwest::Method::GET, &url, None).await?;
    let sha = value
        .get("sha")
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| PublishError::Response("GitHub 文件响应缺少 sha".into()))?
        .to_string();
    let encoded = value.get("content").and_then(Value::as_str).unwrap_or("");
    let compact: String = encoded.chars().filter(|ch| !ch.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(compact)
        .map_err(|error| PublishError::Response(error.to_string()))?;
    let content =
        String::from_utf8(bytes).map_err(|error| PublishError::Response(error.to_string()))?;
    Ok(RemoteFile { sha, content })
}

fn contents_url(repo: &RepoSpec<'_>, path: &str) -> String {
    format!(
        "{}/repos/{}/{}/contents/{}",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo),
        escape_path(path)
    )
}

fn commit_result(value: Value, fallback: &str) -> Result<PutResult, PublishError> {
    let path = value
        .get("content")
        .and_then(|item| item.get("path"))
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .unwrap_or(fallback)
        .to_string();
    let sha = value
        .get("commit")
        .and_then(|item| item.get("sha"))
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| PublishError::Response("GitHub 提交响应缺少 sha".into()))?
        .to_string();
    Ok(PutResult { path, sha })
}

async fn create_file(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    path: &str,
    message: &str,
    content: &str,
) -> Result<PutResult, PublishError> {
    let url = contents_url(repo, path);
    let payload = json!({
        "message": message,
        "content": base64::engine::general_purpose::STANDARD.encode(content.as_bytes()),
        "branch": repo.branch,
    });
    commit_result(
        send(http, repo.token, reqwest::Method::PUT, &url, Some(&payload)).await?,
        path,
    )
}

async fn put_file(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    path: &str,
    message: &str,
    content: &str,
    sha: &str,
    branch: &str,
) -> Result<PutResult, PublishError> {
    let url = format!(
        "{}/repos/{}/{}/contents/{}",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo),
        escape_path(path)
    );
    let payload = json!({
        "message": message,
        "content": base64::engine::general_purpose::STANDARD.encode(content.as_bytes()),
        "branch": branch,
        "sha": sha,
    });
    let value = send(http, repo.token, reqwest::Method::PUT, &url, Some(&payload)).await?;
    let path = value
        .get("content")
        .and_then(|item| item.get("path"))
        .and_then(Value::as_str)
        .unwrap_or(path)
        .to_string();
    let sha = value
        .get("commit")
        .and_then(|item| item.get("sha"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if sha.is_empty() {
        return Err(PublishError::Response("GitHub 提交响应缺少 sha".into()));
    }
    Ok(PutResult { path, sha })
}

async fn branch_sha(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    branch: &str,
) -> Result<String, PublishError> {
    let url = format!(
        "{}/repos/{}/{}/git/ref/heads/{}",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo),
        escape_path(branch)
    );
    let value = send(http, repo.token, reqwest::Method::GET, &url, None).await?;
    value
        .get("object")
        .and_then(|item| item.get("sha"))
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .ok_or_else(|| PublishError::Response("GitHub 分支响应缺少 sha".into()))
}

async fn create_branch(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    branch: &str,
    sha: &str,
) -> Result<(), PublishError> {
    let url = format!(
        "{}/repos/{}/{}/git/refs",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo)
    );
    let payload = json!({"ref": format!("refs/heads/{branch}"), "sha": sha});
    send(
        http,
        repo.token,
        reqwest::Method::POST,
        &url,
        Some(&payload),
    )
    .await?;
    Ok(())
}

async fn create_pull_request(
    http: &reqwest::Client,
    repo: &RepoSpec<'_>,
    title: &str,
    body: &str,
    head: &str,
) -> Result<PullResult, PublishError> {
    let url = format!(
        "{}/repos/{}/{}/pulls",
        repo.api_base.trim_end_matches('/'),
        path_escape(repo.owner),
        path_escape(repo.repo)
    );
    let payload = json!({"title": title, "body": body, "head": head, "base": repo.branch});
    let value = send(
        http,
        repo.token,
        reqwest::Method::POST,
        &url,
        Some(&payload),
    )
    .await?;
    let number = value
        .get("number")
        .and_then(Value::as_i64)
        .ok_or_else(|| PublishError::Response("GitHub PR 响应缺少编号".into()))?;
    let url = value
        .get("html_url")
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| PublishError::Response("GitHub PR 响应缺少地址".into()))?
        .to_string();
    Ok(PullResult { number, url })
}

async fn send(
    http: &reqwest::Client,
    token: &str,
    method: reqwest::Method,
    url: &str,
    payload: Option<&Value>,
) -> Result<Value, PublishError> {
    let mut request = http
        .request(method, url)
        .bearer_auth(token)
        .header(reqwest::header::USER_AGENT, "taozhiyy-api")
        .header(reqwest::header::ACCEPT, "application/vnd.github+json");
    if let Some(payload) = payload {
        request = request.json(payload);
    }
    let response = request
        .send()
        .await
        .map_err(|error| PublishError::Transport(error.to_string()))?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .await
        .map_err(|error| PublishError::Transport(error.to_string()))?;
    if !(200..300).contains(&status) {
        return Err(PublishError::GitHub {
            status,
            message: github_message(&text, status),
        });
    }
    serde_json::from_str(&text).map_err(|error| PublishError::Response(error.to_string()))
}

fn github_message(body: &str, status: u16) -> String {
    let parsed = serde_json::from_str::<Value>(body);
    let message = match parsed {
        Ok(value) => value
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        Err(error) => format!("HTTP {status}: {error}; {}", truncate(body)),
    };
    if message.is_empty() {
        format!("HTTP {status}")
    } else {
        truncate(&message)
    }
}

fn truncate(value: &str) -> String {
    let mut out = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index == 180 {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

fn safe_branch(branch: &str) -> bool {
    !branch.is_empty()
        && !branch.contains("..")
        && !branch.starts_with('/')
        && !branch.ends_with('/')
        && branch
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '/' | '-'))
}

fn escape_path(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .map(path_escape)
        .collect::<Vec<_>>()
        .join("/")
}

fn path_escape(segment: &str) -> String {
    let mut out = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
