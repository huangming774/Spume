use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Multipart, OriginalUri, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::r2::{presign_put, public_url};
use crate::AppState;

/// 和 Go 一致的 JSON 解析失败契约：400 `{"error":"INVALID_JSON", ...}`，
/// 而不是 axum 默认的纯文本 400/422。
pub struct AppJson<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(AppJson(value)),
            Err(_) => Err(fail(
                StatusCode::BAD_REQUEST,
                "INVALID_JSON",
                "请求内容格式不正确。",
            )),
        }
    }
}

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state);
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/auth/me", get(auth_me))
        .route("/api/auth/register", post(auth_register))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/profile", axum::routing::patch(auth_profile))
        .route("/api/auth/verify-security", post(auth_verify_security))
        .route("/api/auth/avatar", post(auth_avatar))
        .route("/api/site/avatar", get(site_avatar))
        .route(
            "/api/guestbook/messages",
            get(guestbook_list).post(guestbook_create),
        )
        .route(
            "/api/guestbook/messages/{id}",
            axum::routing::delete(guestbook_delete).patch(guestbook_patch),
        )
        .route(
            "/api/guestbook/messages/{id}/status",
            axum::routing::patch(guestbook_patch),
        )
        .route("/api/v1/guestbook", get(guestbook_list))
        .route("/api/v1/bangumi/list", get(bangumi_list))
        .route("/api/v1/radar/feed", get(radar_feed))
        .route("/api/v1/bili/videos", get(bili_videos))
        .route("/api/v1/acg/image/{name}", get(acg_image))
        .route("/api/v1/wallpapers/draw", get(wallpaper_draw))
        .route("/api/v1/search", get(search))
        .route("/api/v1/sync/trigger", post(sync_trigger))
        .route("/api/chat", get(chat_quota).post(chat_post))
        .route("/api/chat/stats", get(chat_stats))
        .route("/api/ai/image", get(image_quota).post(image_create))
        .route("/api/ai/image/gallery", get(image_gallery))
        .route("/api/server/info", get(server_info))
        .route("/api/client/network", get(client_network))
        .route("/api/client/place", get(crate::weather::place))
        .route(
            "/api/client/map-tile/{layer}/{z}/{x}/{y}",
            get(crate::map_tile::tile),
        )
        .route("/api/weather", get(crate::weather::get))
        .route("/api/client/deploy-status", get(client_deploy_status))
        .route("/api/client/music", get(client_music))
        .route("/api/client/music/playlist", get(client_music_playlist))
        .route("/api/owner/music", axum::routing::put(owner_music_save))
        .route("/api/content", get(content_recent))
        .route("/api/content/{kind}", get(content_list))
        .route("/api/owner/status", get(owner_status))
        .route("/api/owner/emails", get(owner_emails))
        .route(
            "/api/owner/drafts",
            get(owner_drafts).post(owner_draft_create),
        )
        .route("/api/owner/publish", post(owner_publish))
        .route("/api/owner/content/{id}", delete(owner_content_delete))
        .route("/api/owner/posts", delete(owner_post_delete))
        .route(
            "/api/owner/journal",
            post(owner_journal).delete(owner_journal_delete),
        )
        .route(
            "/api/owner/gallery/images",
            post(owner_gallery).delete(owner_gallery_delete),
        )
        .route("/api/owner/friends", post(owner_friends))
        .route("/api/owner/friends/approve", post(owner_friend_approve))
        .route(
            "/api/owner/moments",
            post(owner_moments).delete(owner_moment_delete),
        )
        .route(
            "/api/owner/about",
            post(owner_about).delete(owner_about_delete),
        )
        .route(
            "/api/owner/bili",
            post(owner_bili).delete(owner_bili_delete),
        )
        .route(
            "/api/owner/projects",
            post(owner_project).delete(owner_project_delete),
        )
        .route(
            "/api/owner/ai/fixed-answers",
            get(fixed_answers).post(fixed_answer_create),
        )
        .route("/api/owner/ai/generate", post(owner_ai_generate))
        .route(
            "/api/owner/notifications/{id}/read",
            axum::routing::patch(notification_read),
        )
        .route("/api/owner/uploads", post(owner_upload))
        .route("/api/owner/uploads/presign", post(owner_presign))
        .route("/api/owner/assets", post(owner_asset))
        .route("/api/internal/sync/bili", post(sync_bili))
        .route("/api/internal/sync/radar", post(sync_radar))
        .route("/api/internal/sync/wallpaper", post(sync_wallpaper))
        .route("/api/internal/maintenance", post(maintain))
        .route("/api/internal/index-content", post(index_site))
        .route("/api/internal/refresh-ai-model", post(refresh_ai_model))
        .route("/api/internal/jobs/image/{id}", post(restock_image))
        .route("/api/internal/publish", post(internal_publish))
        .route("/api/internal/mail", post(internal_mail))
        .route("/api/internal/reindex", post(internal_reindex))
        .route("/api/client/view", post(client_view))
        .route("/api/client/hot", get(client_hot))
        .route("/api/client/image-search", get(client_image_search))
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(axum::middleware::from_fn(inject_remote_ip))
        .with_state(state);
    match cors {
        Some(cors) => app.layer(cors),
        None => app,
    }
}

/// 没有 X-Forwarded-For / X-Real-IP 时，用 TCP 连接的远端地址回填，
/// 和 Go 的 clientIP 回退 RemoteAddr 一致，避免直连时全站共用一个限流桶。
/// 连接信息缺失（如单测直呼 router）时跳过。
async fn inject_remote_ip(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    // 空的代理头和 Go 的 clientIP 一样当没有：X-Forwarded-For / X-Real-IP 都为空时
    // 依然回填直连地址，否则全站会共用一个限流桶。
    let has_proxy_header = header_text(request.headers(), "x-forwarded-for")
        .or_else(|| header_text(request.headers(), "x-real-ip"))
        .is_some();
    if !has_proxy_header {
        if let Some(axum::extract::ConnectInfo(addr)) =
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        {
            if let Ok(value) = HeaderValue::from_str(&addr.ip().to_string()) {
                request.headers_mut().insert("x-real-ip", value);
            }
        }
    }
    next.run(request).await
}

/// 来源列表为空时返回 None：整个不加 CORS 层，和 Go 版一致——
/// Go 只在来源命中白名单时才回 `Access-Control-Allow-Origin`，白名单为空时
/// 任何请求都拿不到 CORS 头（此时不能用 `AllowOrigin::any()` 加 credentials）。
fn cors_layer(state: &AppState) -> Option<CorsLayer> {
    let origins: Vec<HeaderValue> = state
        .cfg
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    if origins.is_empty() {
        return None;
    }
    Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::AUTHORIZATION,
                header::COOKIE,
                HeaderName::from_static("x-blog-user-id"),
                HeaderName::from_static("x-sync-trigger-token"),
            ]),
    )
}

pub(crate) fn fail(status: StatusCode, error: &str, message: &str) -> Response {
    (status, Json(json!({"error": error, "message": message}))).into_response()
}

fn query_failed(error: &sqlx::Error) -> Response {
    // 原始错误（可能含 SQL、连接串）只进日志，不给客户端。
    tracing::error!(error = %error, "database query failed");
    fail(
        StatusCode::INTERNAL_SERVER_ERROR,
        "SERVER_ERROR",
        "服务器开小差了，请稍后再试",
    )
}

fn db_required(state: &AppState) -> Result<&sqlx::PgPool, Box<Response>> {
    state.db.as_ref().ok_or_else(|| {
        Box::new(fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        ))
    })
}

#[derive(Clone)]
struct SessionUser {
    id: i64,
    email: String,
    display_name: String,
    avatar: String,
    is_owner: bool,
    unlimited: bool,
    created_at: DateTime<Utc>,
}

async fn session_user(state: &AppState, headers: &HeaderMap) -> Option<SessionUser> {
    let pool = state.db.as_ref()?;
    let token = session_token(headers)?;
    sqlx::query_as::<_, (i64, String, String, String, bool, bool, DateTime<Utc>)>(
        r#"SELECT u.id, u.email, u.display_name, u.avatar, u.is_owner, s.unlimited, u.created_at
           FROM sessions s JOIN users u ON u.id = s.user_id
           WHERE s.id = $1 AND s.expires_at > now()"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(
        |(id, email, display_name, avatar, is_owner, unlimited, created_at)| SessionUser {
            id,
            email,
            display_name,
            avatar,
            is_owner,
            unlimited,
            created_at,
        },
    )
}

fn session_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().strip_prefix("acg_session="))
        .next()
}

fn session_cookie(token: &str, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "acg_session={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000{secure}"
    ))
    .expect("cookie")
}

fn clear_cookie(secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "acg_session=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{secure}"
    ))
    .expect("cookie")
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    if let Some(body) = fresh_health(&state) {
        return Json(body);
    }
    if let Some((body, false)) = crate::cache::get(&state, "health:snapshot").await {
        return Json(body);
    }
    let body = health_snapshot(&state).await;
    store_health(&state, body.clone());
    Json(body)
}

pub(crate) fn fresh_health(state: &AppState) -> Option<Value> {
    let guard = state.health_cache.lock().ok()?;
    let (fetched_at, body) = guard.as_ref()?;
    if fetched_at.elapsed() < state.cfg.timeouts.health_refresh {
        Some(body.clone())
    } else {
        None
    }
}

pub(crate) fn store_health(state: &AppState, body: Value) {
    if let Ok(mut guard) = state.health_cache.lock() {
        *guard = Some((Instant::now(), body.clone()));
    }
    // 共享缓存：其它实例冷启动也能直接读到新鲜快照。
    let state = state.clone();
    tokio::spawn(async move {
        crate::cache::set(
            &state,
            "health:snapshot",
            &body,
            false,
            state.cfg.timeouts.health_refresh,
        )
        .await;
    });
}

pub(crate) async fn health_snapshot(state: &AppState) -> Value {
    let db_check = async {
        match &state.db {
            Some(pool) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
            None => false,
        }
    };
    let db = tokio::time::timeout(Duration::from_secs(2), db_check)
        .await
        .unwrap_or_default();
    let redis_url = if state.cfg.redis_ready() {
        format!("{}/ping", state.cfg.redis_url.trim_end_matches('/'))
    } else {
        String::new()
    };
    let r2_url = if state.cfg.r2_ready() {
        state.cfg.r2_public_base.clone()
    } else {
        String::new()
    };
    let probe_limit = Duration::from_secs(3);
    let (redis, vector, r2) = tokio::join!(
        tokio::time::timeout(
            probe_limit,
            crate::jobs::probe(state, &redis_url, &state.cfg.redis_token, false),
        ),
        tokio::time::timeout(probe_limit, crate::jobs::vector_probe(state)),
        tokio::time::timeout(probe_limit, crate::jobs::probe(state, &r2_url, "", true)),
    );
    let redis = redis.unwrap_or("down");
    let vector = vector.unwrap_or("down");
    let r2 = r2.unwrap_or("down");
    let ready = db && redis != "down" && vector != "down" && r2 != "down";
    json!({
        "status": if ready { "ok" } else { "degraded" },
        "uid": state.cfg.bilibili_uid,
        "db": if db { "up" } else { "down" },
        "redis": redis,
        "r2": r2,
        "vector": vector,
    })
}

#[derive(Deserialize)]
struct AuthBody {
    #[serde(default)]
    email: String,
    #[serde(default)]
    password: String,
}

async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> Json<Value> {
    let limit = state.cfg.timeouts.session;
    let started = Instant::now();
    let user = match tokio::time::timeout(limit, session_user(&state, &headers)).await {
        Ok(user) => user,
        Err(_) => {
            tracing::warn!(
                elapsed_ms = started.elapsed().as_millis() as u64,
                deadline = "critical",
                "session lookup timed out"
            );
            None
        }
    };
    let Some(user) = user else {
        return Json(json!({"loggedIn": false}));
    };
    Json(json!({
        "loggedIn": true,
        "unlimited": user.unlimited,
        "user": user_json(&user),
    }))
}

async fn auth_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<AuthBody>,
) -> Response {
    if let Some(response) = auth_limited(&state, &headers) {
        return response;
    }
    let email = body.email.trim().to_lowercase();
    if !state.cfg.owner_email.is_empty() && email == state.cfg.owner_email {
        return fail(
            StatusCode::FORBIDDEN,
            "OWNER_EMAIL_RESERVED",
            "该邮箱为站长专用，请直接登录并完成学号验证",
        );
    }
    if !crate::account::valid_email(&email) {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_EMAIL",
            "请输入有效的邮箱地址",
        );
    }
    if let Some(message) = crate::account::password_message(&body.password) {
        return fail(StatusCode::BAD_REQUEST, "INVALID_PASSWORD", message);
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let hash = match bcrypt::hash(&body.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "密码处理失败",
            )
        }
    };
    let inserted = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO users (email, password_hash, display_name, is_owner) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&email)
    .bind(&hash)
    .bind(crate::account::nickname_from_email(&email))
    .bind(false)
    .fetch_one(pool)
    .await;
    let (id,) = match inserted {
        Ok(row) => row,
        Err(error) => {
            let unique = error
                .as_database_error()
                .and_then(|item| item.code())
                .is_some_and(|code| code == "23505");
            if unique {
                return fail(
                    StatusCode::CONFLICT,
                    "EMAIL_EXISTS",
                    "该邮箱已注册，请直接登录",
                );
            }
            return query_failed(&error);
        }
    };
    issue_session(&state, pool, id, false, &headers).await
}

async fn auth_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<AuthBody>,
) -> Response {
    if let Some(response) = auth_limited(&state, &headers) {
        return response;
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    // 不带 @ 的输入等于站长登录名时，映射到站长邮箱；否则按邮箱处理。
    let email = crate::account::resolve_login_email(
        &body.email,
        &state.cfg.owner_login_name,
        &state.cfg.owner_email,
    );
    let row = sqlx::query_as::<_, (i64, String, String, String, bool)>(
        "SELECT id, password_hash, display_name, avatar, is_owner FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await;
    let (id, hash, _display_name, _avatar, owner) = match row {
        Ok(Some(values)) => values,
        Ok(None) => {
            return fail(
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "账号或密码错误",
            )
        }
        Err(error) => return query_failed(&error),
    };
    if bcrypt::verify(&body.password, &hash).ok() != Some(true) {
        if owner {
            if let Err(response) = note_audit(
                &state,
                &headers,
                AuditNote {
                    event: "owner.login",
                    outcome: "failure",
                    actor: 0,
                    target_type: "owner",
                    target_id: "",
                    detail: "invalid_credentials",
                },
            )
            .await
            {
                return *response;
            }
        }
        return fail(
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "账号或密码错误",
        );
    }
    if owner {
        // 站长账号密码直接进后台，不再走安全问答（有意去掉双重验证）。
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "owner.login",
                outcome: "success",
                actor: id,
                target_type: "owner",
                target_id: &id.to_string(),
                detail: "password_login",
            },
        )
        .await
        {
            return *response;
        }
    }
    issue_session(&state, pool, id, owner, &headers).await
}

async fn issue_session(
    state: &AppState,
    pool: &sqlx::PgPool,
    user_id: i64,
    owner: bool,
    headers: &HeaderMap,
) -> Response {
    let token = hex::encode(rand::random::<[u8; 32]>());
    let saved = sqlx::query("INSERT INTO sessions (id, user_id, unlimited, expires_at) VALUES ($1, $2, $3, now() + interval '30 days')")
        .bind(&token)
        .bind(user_id)
        .bind(owner)
        .execute(pool)
        .await;
    if saved.is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "会话创建失败",
        );
    }
    let user = session_user(
        state,
        &HeaderMap::from_iter([(
            header::COOKIE,
            HeaderValue::from_str(&format!("acg_session={token}")).unwrap(),
        )]),
    )
    .await;
    let mut response = Json(json!({"ok": true, "loggedIn": true, "unlimited": owner, "user": user.as_ref().map(user_json)})).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        session_cookie(&token, cookie_secure(state, headers)),
    );
    response
}

/// Go 的 cookieSecure：默认安全，除非显式关掉；HTTPS（含反代）时总是安全。
fn cookie_secure(state: &AppState, headers: &HeaderMap) -> bool {
    if !state.cfg.cookie_secure {
        return false;
    }
    header_text(headers, "x-forwarded-proto")
        .is_some_and(|value| value.eq_ignore_ascii_case("https"))
        || header_text(headers, header::ORIGIN.as_str())
            .is_some_and(|value| value.starts_with("https://"))
}

async fn auth_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let (Some(pool), Some(token)) = (state.db.as_ref(), session_token(&headers)) {
        // Go 忽略删除失败，注销总是成功。
        if let Err(error) = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(token)
            .execute(pool)
            .await
        {
            tracing::warn!(%error, "session delete failed during logout");
        }
    }
    let mut response = Json(json!({"ok": true})).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        clear_cookie(cookie_secure(&state, &headers)),
    );
    response
}

#[derive(Deserialize)]
struct ProfileBody {
    #[serde(rename = "displayName", default)]
    display_name: String,
}

async fn auth_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ProfileBody>,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    let name = body.display_name.trim();
    if let Some(message) = crate::account::display_name_message(name) {
        return fail(StatusCode::BAD_REQUEST, "INVALID_DISPLAY_NAME", message);
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    if let Err(error) = sqlx::query("UPDATE users SET display_name = $1 WHERE id = $2")
        .bind(name)
        .bind(user.id)
        .execute(pool)
        .await
    {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            &format!("昵称没有保存: {error}"),
        );
    }
    // 与头像同理：公开读取落在最早的站长行，昵称也同步到所有站长行。
    if user.is_owner {
        if let Err(error) = sqlx::query("UPDATE users SET display_name = $1 WHERE is_owner = true")
            .bind(name)
            .execute(pool)
            .await
        {
            tracing::warn!(error = %error, "owner display name fan-out failed");
        }
    }
    crate::cache::del(&state, &["site:profile"]).await;
    let mut updated = user;
    updated.display_name = name.to_string();
    Json(json!({"ok": true, "user": user_json(&updated)})).into_response()
}

#[derive(Deserialize)]
struct VerifyBody {
    #[serde(rename = "challengeToken", default)]
    challenge_token: String,
    #[serde(default)]
    answer: String,
}

async fn auth_verify_security(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<VerifyBody>,
) -> Response {
    if let Some(response) = auth_limited(&state, &headers) {
        return response;
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let token = body.challenge_token.trim();
    let user_id = if token.is_empty() {
        None
    } else {
        match sqlx::query_scalar::<_, i64>(
            "DELETE FROM login_challenges WHERE token = $1 AND expires_at > now() RETURNING user_id",
        )
        .bind(token)
        .fetch_optional(pool)
        .await
        {
            Ok(value) => value,
            Err(error) => return query_failed(&error),
        }
    };
    let Some(user_id) = user_id else {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "owner.security_verify",
                outcome: "failure",
                actor: 0,
                target_type: "owner",
                target_id: "",
                detail: "invalid_challenge",
            },
        )
        .await
        {
            return *response;
        }
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_CHALLENGE",
            "验证已失效，请重新登录",
        );
    };
    let is_owner = sqlx::query_scalar::<_, bool>("SELECT is_owner FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await;
    let is_owner = match is_owner {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(error) => return query_failed(&error),
    };
    if !is_owner {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "owner.security_verify",
                outcome: "failure",
                actor: user_id,
                target_type: "owner",
                target_id: &user_id.to_string(),
                detail: "forbidden_user",
            },
        )
        .await
        {
            return *response;
        }
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "无权进行此验证");
    }
    if body.answer.trim() != state.cfg.owner_security_answer.trim() {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "owner.security_verify",
                outcome: "failure",
                actor: user_id,
                target_type: "owner",
                target_id: &user_id.to_string(),
                detail: "wrong_answer",
            },
        )
        .await
        {
            return *response;
        }
        return fail(StatusCode::UNAUTHORIZED, "WRONG_ANSWER", "学号回答不正确");
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "owner.security_verify",
            outcome: "success",
            actor: user_id,
            target_type: "owner",
            target_id: &user_id.to_string(),
            detail: "ok",
        },
    )
    .await
    {
        return *response;
    }
    issue_session(&state, pool, user_id, true, &headers).await
}

async fn auth_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    if let Some(response) = auth_limited(&state, &headers) {
        return response;
    }
    let (public, _key, _name, _size, _mime) = match store_upload(&state, multipart, "avatars").await
    {
        Ok(saved) => saved,
        Err(response) => return *response,
    };
    let Some(pool) = state.db.as_ref() else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_UNAVAILABLE",
            "头像已上传，但数据库还没有连上",
        );
    };
    if let Err(error) = sqlx::query("UPDATE users SET avatar = $1 WHERE id = $2")
        .bind(&public)
        .bind(user.id)
        .execute(pool)
        .await
    {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            &format!("头像已上传，资料没有更新: {error}"),
        );
    }
    // 首页/AI 读的是「最早的 is_owner 行」，历史数据可能有多行站长或登录行非站长，
    // 这里把头像同步到所有站长行，保证上传后全站立刻一致。
    if user.is_owner {
        if let Err(error) = sqlx::query("UPDATE users SET avatar = $1 WHERE is_owner = true")
            .bind(&public)
            .execute(pool)
            .await
        {
            tracing::warn!(error = %error, "owner avatar fan-out failed");
        }
    }
    crate::cache::del(&state, &["site:profile"]).await;
    Json(json!({"ok": true, "user": {"id": user.id, "email": user.email, "displayName": crate::account::display_name_or_email(&user.email, &user.display_name), "avatar": public, "isOwner": user.is_owner}})).into_response()
}

async fn site_avatar(State(state): State<AppState>) -> Json<Value> {
    if let Some((body, false)) = crate::cache::get(&state, "site:profile").await {
        return Json(body);
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"avatar": "", "displayName": ""}));
    };
    let row = match sqlx::query_as::<_, (String, String)>(
        "SELECT avatar, COALESCE(display_name, '') FROM users WHERE is_owner = true ORDER BY id ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    {
        Ok(row) => row,
        Err(error) => {
            tracing::error!(error = %error, "site avatar lookup failed");
            None
        }
    };
    let (avatar, display_name) = match row {
        Some((avatar, name)) => {
            let avatar = if avatar.starts_with("http://") || avatar.starts_with("https://") {
                avatar
            } else {
                String::new()
            };
            (avatar, name)
        }
        None => (String::new(), String::new()),
    };
    let body = json!({"avatar": avatar, "displayName": display_name});
    crate::cache::set(
        &state,
        "site:profile",
        &body,
        false,
        Duration::from_secs(60),
    )
    .await;
    Json(body)
}

const MUSIC_SETTING_KEY: &str = "netease_playlist";

/// 读站长配置的网易云歌单。返回 `{enabled, playlistId, raw}`；没配过 enabled=false。
async fn music_setting(state: &AppState) -> (bool, String, String) {
    let Some(pool) = state.db.as_ref() else {
        return (false, String::new(), String::new());
    };
    let raw: Option<String> = sqlx::query_scalar("SELECT value FROM site_settings WHERE key = $1")
        .bind(MUSIC_SETTING_KEY)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
    let raw = raw.unwrap_or_default();
    let id = crate::music::parse_playlist_id(&raw).unwrap_or_default();
    (!id.is_empty(), id, raw)
}

/// `GET /api/client/music`：公开的音乐配置，前端据此决定播歌单还是回退 loop.mp3。
async fn client_music(State(state): State<AppState>) -> Json<Value> {
    let (enabled, playlist_id, raw) = music_setting(&state).await;
    Json(json!({"enabled": enabled, "playlistId": playlist_id, "raw": raw}))
}

/// `GET /api/client/music/playlist`：解析后的播放列表。
/// 没配置返回 404，上游失败返回 502，前端两种情况都回退 loop.mp3。
async fn client_music_playlist(State(state): State<AppState>) -> Response {
    let (enabled, playlist_id, _) = music_setting(&state).await;
    if !enabled {
        return fail(
            StatusCode::NOT_FOUND,
            "MUSIC_NOT_CONFIGURED",
            "还没有配置歌单",
        );
    }
    match crate::music::resolve(&state, &playlist_id).await {
        Ok(body) => Json(body).into_response(),
        Err(error) => {
            tracing::warn!(%error, playlist = %playlist_id, "netease playlist resolve failed");
            fail(
                StatusCode::BAD_GATEWAY,
                "MUSIC_UNAVAILABLE",
                "歌单暂时解析不了",
            )
        }
    }
}

#[derive(Deserialize)]
struct MusicBody {
    #[serde(default)]
    playlist: String,
}

/// `PUT /api/owner/music`：站长保存歌单链接/ID。传空串等于停用。
async fn owner_music_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<MusicBody>,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let raw = body.playlist.trim();
    if !raw.is_empty() && crate::music::parse_playlist_id(raw).is_none() {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_PLAYLIST",
            "请填网易云歌单链接或数字 ID",
        );
    }
    let Some(pool) = state.db.as_ref() else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    if let Err(error) = sqlx::query(
        "INSERT INTO site_settings (key, value, updated_at) VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind(MUSIC_SETTING_KEY)
    .bind(raw)
    .execute(pool)
    .await
    {
        return query_failed(&error);
    }
    if let Some(id) = crate::music::parse_playlist_id(raw) {
        crate::music::invalidate(&state, &id).await;
    }
    let (enabled, playlist_id, raw) = music_setting(&state).await;
    Json(json!({"ok": true, "enabled": enabled, "playlistId": playlist_id, "raw": raw}))
        .into_response()
}

/// Go 接口里的时间串：RFC3339 秒精度、Z 结尾（`time.Now().UTC().Format(time.RFC3339)`）。
fn go_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn user_json(user: &SessionUser) -> Value {
    json!({
        "id": user.id,
        "email": user.email,
        "displayName": crate::account::display_name_or_email(&user.email, &user.display_name),
        "avatar": user.avatar,
        "isOwner": user.is_owner,
        "createdAt": go_timestamp(user.created_at),
    })
}

#[derive(Deserialize)]
struct PageQuery {
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
    channel: Option<String>,
}

async fn guestbook_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let is_admin = session_user(&state, &headers)
        .await
        .is_some_and(|user| user.is_owner);
    let Some(channel) = crate::guestbook::normalize_channel(query.channel.as_deref().unwrap_or(""))
    else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_CHANNEL", "留言分区不正确");
    };
    let page = query.page.unwrap_or(1).max(1);
    let page_size = crate::guestbook::accepted_page_size(query.page_size);
    let Ok(pool) = db_required(&state) else {
        return Json(json!({"items": [], "page": page, "pageSize": page_size, "total": 0, "channel": channel, "isAdmin": is_admin})).into_response();
    };
    match crate::guestbook::load_page(pool, channel, page, page_size, is_admin).await {
        Ok((items, total)) => Json(json!({"items": items, "page": page, "pageSize": page_size, "total": total, "channel": channel, "isAdmin": is_admin})).into_response(),
        Err(_) => fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "加载留言失败",
        ),
    }
}

#[derive(Deserialize)]
struct GuestbookBody {
    nickname: Option<String>,
    #[serde(default)]
    content: String,
    channel: Option<String>,
    #[serde(rename = "contactEmail")]
    contact_email: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: Option<i64>,
    #[serde(default)]
    extra: Option<Value>,
}

async fn guestbook_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<GuestbookBody>,
) -> Response {
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let Some(channel) = crate::guestbook::normalize_channel(body.channel.as_deref().unwrap_or(""))
        .map(str::to_string)
    else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_CHANNEL", "留言分区不正确");
    };
    let parent_id = body.parent_id.unwrap_or(0);
    // 检查顺序对齐 Go：分区 -> 维护 -> 正文 -> 楼层（guestbook_messages.go:312-341）。
    if channel == "guestbook" && !state.cfg.guestbook_main_posts {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "GUESTBOOK_DISABLED",
            "留言区暂时维护中，已暂停提交。",
        );
    }
    let content = body.content.trim();
    if let Err(error) = crate::guestbook::check_content(content) {
        return fail(StatusCode::BAD_REQUEST, error.code, error.message);
    }
    if parent_id < 0 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_PARENT", "回复楼层不正确");
    }
    let user = session_user(&state, &headers).await;
    let admin = user.as_ref().is_some_and(|item| item.is_owner);
    if parent_id > 0 {
        let visible = if admin {
            "status IN ('visible', 'hidden')"
        } else {
            "status = 'visible'"
        };
        let count: i64 = match sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM guestbook_messages WHERE id = $1 AND channel = $2 AND {visible}"
        )))
        .bind(parent_id)
        .bind(&channel)
        .fetch_one(pool)
        .await
        {
            Ok(count) => count,
            Err(error) => {
                // Go 在核对失败时当作不存在（guestbook_messages.go:578-589）。
                tracing::warn!(%error, "guestbook parent check failed");
                0
            }
        };
        if count == 0 {
            return fail(
                StatusCode::BAD_REQUEST,
                "INVALID_PARENT",
                "要回复的留言不存在或不可见",
            );
        }
    }
    let composed = match crate::guestbook::compose(crate::guestbook::ComposeInput {
        channel: &channel,
        parent_id,
        nickname: body.nickname.as_deref().unwrap_or(""),
        email: body.contact_email.as_deref().unwrap_or(""),
        account_name: user.as_ref().map(|item| item.display_name.as_str()),
        account_email: user.as_ref().map(|item| item.email.as_str()),
        account_avatar: user.as_ref().map(|item| item.avatar.as_str()).unwrap_or(""),
    }) {
        Ok(composed) => composed,
        Err(error) => return fail(StatusCode::BAD_REQUEST, error.code, error.message),
    };
    let ip = client_ip(&headers);
    let user_agent = header_text(&headers, header::USER_AGENT.as_str()).unwrap_or_default();
    let ip_hash = crate::guestbook::hash_key("ip", &ip);
    let user_agent_hash = crate::guestbook::hash_key("ua", &user_agent);
    let content_hash = crate::guestbook::hash_key("content", content);
    if let Err(response) = guestbook_rate(
        pool,
        user.as_ref().map(|item| item.id),
        &ip_hash,
        &user_agent_hash,
    )
    .await
    {
        return *response;
    }
    // Redis SETNX 60s 防连点，比查库便宜；未配置/出错回退 DB 去重。
    let dedupe_key = format!("dedupe:gb:{ip_hash}:{content_hash}");
    match crate::cache::set_nx(&state, &dedupe_key, Duration::from_secs(60)).await {
        Some(false) => {
            return fail(
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "相同内容刚刚提交过，请稍后再试。",
            );
        }
        Some(true) => {}
        None => {
            if let Err(response) = guestbook_duplicate(
                pool,
                user.as_ref().map(|item| item.id),
                &ip_hash,
                &content_hash,
            )
            .await
            {
                return *response;
            }
        }
    }
    let region = match crate::guestbook::lookup_region(Some(pool), &state.http, &ip).await {
        Ok(region) => region,
        Err(error) => {
            // Go 的 lookupIPRegion 从不失败，查不到就记「未知地区」继续提交（guestbook_ip.go:50-71）。
            tracing::warn!(%error, "guestbook region lookup failed");
            "未知地区".to_string()
        }
    };
    // 友链申请的结构化字段：只收白名单键、限长、链接必须公开 http(s)。
    let mut extra = json!({});
    if channel == "friends" && parent_id == 0 {
        if let Some(map) = body.extra.as_ref().and_then(Value::as_object) {
            let mut clean = serde_json::Map::new();
            for key in ["name", "desc", "url", "avatar"] {
                let Some(text) = map.get(key).and_then(Value::as_str).map(str::trim) else {
                    continue;
                };
                if text.is_empty() || text.chars().count() > 300 {
                    continue;
                }
                if matches!(key, "url" | "avatar") && !public_http_url(text) {
                    continue;
                }
                clean.insert(key.to_string(), json!(text));
            }
            if !clean.is_empty() {
                extra = Value::Object(clean);
            }
        }
    }
    let extra_text = serde_json::to_string(&extra).unwrap_or_else(|_| "{}".to_string());
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>)>(
        "INSERT INTO guestbook_messages (user_id, nickname, avatar, channel, content, contact_email, content_hash, ip_hash, ip_region, ip_masked, user_agent_hash, parent_id, is_login_user, is_admin_user, extra) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15::jsonb) RETURNING id, created_at",
    )
    .bind(user.as_ref().map(|item| item.id))
    .bind(&composed.nickname)
    .bind(&composed.avatar)
    .bind(&channel)
    .bind(content)
    .bind(&composed.email)
    .bind(&content_hash)
    .bind(&ip_hash)
    .bind(&region)
    .bind(crate::guestbook::mask_ip(&ip))
    .bind(&user_agent_hash)
    .bind(parent_id)
    .bind(user.is_some())
    .bind(admin)
    .bind(&extra_text)
    .fetch_one(pool)
    .await;
    let Ok((id, created)) = row else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "小纸条没有贴上，请稍后再试哦",
        );
    };
    let nickname = composed.nickname;
    // Go 用 RFC3339 秒精度、Z 结尾（guestbook_messages.go:422）。
    let created_at = created.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mail_payload = json!({
        "channel": channel,
        "nickname": nickname,
        "content": content,
        "createdAt": created_at,
        "reply": parent_id > 0,
        "adminReply": admin && parent_id > 0,
        "parentId": parent_id,
    });
    let mail_state = state.clone();
    let mail_payload2 = mail_payload.clone();
    enqueue_or_spawn(&state, "/api/internal/mail", mail_payload, async move {
        run_mail_job(&mail_state, &mail_payload2).await;
    })
    .await;
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "guestbook.create",
            outcome: "success",
            actor: user.as_ref().map(|item| item.id).unwrap_or(0),
            target_type: "guestbook",
            target_id: &id.to_string(),
            detail: channel.as_str(),
        },
    )
    .await
    {
        return *response;
    }
    let item = crate::guestbook::StoredMessage {
        id,
        parent_id,
        nickname,
        avatar: composed.avatar,
        content: content.to_string(),
        ip_region: region,
        is_login_user: user.is_some(),
        is_admin_user: admin,
        status: "visible".into(),
        ip_masked: String::new(),
        user_agent_hash: String::new(),
        created_at: created,
        extra,
    };
    let Some(mut item) = crate::guestbook::thread_json(vec![item], Vec::new(), false)
        .into_iter()
        .next()
    else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "小纸条没有贴上，请稍后再试哦",
        );
    };
    // Go 的新增响应总是带回 status（guestbook_messages.go:436-447）。
    item["status"] = json!("visible");
    Json(json!({"message": "留言成功", "item": item})).into_response()
}

async fn guestbook_rate(
    pool: &sqlx::PgPool,
    user_id: Option<i64>,
    ip_hash: &str,
    user_agent_hash: &str,
) -> Result<(), Box<Response>> {
    let now = Utc::now();
    let hour_ago = now - chrono::Duration::hours(1);
    let day_start = now.date_naive().and_time(chrono::NaiveTime::MIN).and_utc();
    let counts = if let Some(user_id) = user_id {
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM guestbook_messages WHERE user_id = $1 AND created_at >= $2), (SELECT COUNT(*) FROM guestbook_messages WHERE user_id = $1 AND created_at >= $3)",
        )
        .bind(user_id)
        .bind(hour_ago)
        .bind(day_start)
        .fetch_one(pool)
        .await
        .map(|counts| (counts, 10_i64, 30_i64))
    } else {
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM guestbook_messages WHERE ip_hash = $1 AND user_agent_hash = $2 AND is_login_user = false AND created_at >= $3), (SELECT COUNT(*) FROM guestbook_messages WHERE ip_hash = $1 AND user_agent_hash = $2 AND is_login_user = false AND created_at >= $4)",
        )
        .bind(ip_hash)
        .bind(user_agent_hash)
        .bind(hour_ago)
        .bind(day_start)
        .fetch_one(pool)
        .await
        .map(|counts| (counts, 3_i64, 10_i64))
    };
    let ((hour_count, day_count), hour_limit, day_limit) = match counts {
        Ok(counts) => counts,
        Err(error) => {
            // Go 在核对失败时放行，只留告警。
            tracing::warn!(%error, "guestbook rate check failed; allowing");
            return Ok(());
        }
    };
    if hour_count >= hour_limit || day_count >= day_limit {
        return Err(Box::new(fail(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "留言太频繁啦，稍后再来贴小纸条吧～",
        )));
    }
    Ok(())
}

async fn guestbook_duplicate(
    pool: &sqlx::PgPool,
    user_id: Option<i64>,
    ip_hash: &str,
    content_hash: &str,
) -> Result<(), Box<Response>> {
    let since = Utc::now() - chrono::Duration::seconds(60);
    let count = if let Some(user_id) = user_id {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM guestbook_messages WHERE user_id = $1 AND content_hash = $2 AND created_at >= $3",
        )
        .bind(user_id)
        .bind(content_hash)
        .bind(since)
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM guestbook_messages WHERE ip_hash = $1 AND content_hash = $2 AND is_login_user = false AND created_at >= $3",
        )
        .bind(ip_hash)
        .bind(content_hash)
        .bind(since)
        .fetch_one(pool)
        .await
    };
    match count {
        Ok(0) => Ok(()),
        Ok(_) => Err(Box::new(fail(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "相同内容提交太快啦，稍后再试试～",
        ))),
        Err(error) => {
            // Go 在核对失败时放行。
            tracing::warn!(%error, "guestbook duplicate check failed; allowing");
            Ok(())
        }
    }
}

async fn guestbook_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(raw_id): Path<String>,
) -> Response {
    // Go 先解析 id，非法或 <= 0 直接 404（guestbook_messages.go:70-73）。
    let Ok(id) = raw_id.parse::<i64>() else {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    };
    if id <= 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    }
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作");
    };
    if !user.is_owner {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作");
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let updated = match sqlx::query(
        "UPDATE guestbook_messages SET status = 'deleted' WHERE id = $1 AND status <> 'deleted'",
    )
    .bind(id)
    .execute(pool)
    .await
    {
        Ok(updated) => updated,
        Err(error) => {
            tracing::warn!(%error, "guestbook delete failed");
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "删除失败",
            );
        }
    };
    if updated.rows_affected() == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "guestbook.delete",
            outcome: "success",
            actor: user.id,
            target_type: "guestbook_message",
            target_id: &id.to_string(),
            detail: "deleted",
        },
    )
    .await
    {
        return *response;
    }
    Json(json!({"ok": true, "message": "已删除"})).into_response()
}

async fn guestbook_patch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(raw_id): Path<String>,
    AppJson(body): AppJson<Value>,
) -> Response {
    // Go 先解析 id，非法或 <= 0 直接 404（guestbook_messages.go:56-62）。
    let Ok(id) = raw_id.parse::<i64>() else {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    };
    if id <= 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    }
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作");
    };
    if !user.is_owner {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "无权操作");
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    // Go 会 trim 后再比对（guestbook_messages.go:686-691）。
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !matches!(status, "visible" | "hidden" | "deleted") {
        return fail(StatusCode::BAD_REQUEST, "INVALID_CONTENT", "无效的状态");
    }
    let updated = match sqlx::query("UPDATE guestbook_messages SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(updated) => updated,
        Err(error) => {
            tracing::warn!(%error, "guestbook status update failed");
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "更新失败",
            );
        }
    };
    if updated.rows_affected() == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在");
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "guestbook.status_change",
            outcome: "success",
            actor: user.id,
            target_type: "guestbook_message",
            target_id: &id.to_string(),
            detail: status,
        },
    )
    .await
    {
        return *response;
    }
    Json(json!({"ok": true, "status": status})).into_response()
}

async fn bangumi_list(State(state): State<AppState>) -> Response {
    match list_simple(&state, "SELECT id, title, watched, total, latest_episode, cover_path, link_url FROM bangumi_items ORDER BY title").await {
        Ok(items) => Json(json!({"items": items})).into_response(),
        Err(error) => query_failed(&error),
    }
}

async fn radar_feed(State(state): State<AppState>) -> Response {
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    let rows = match sqlx::query_as::<_, (String, String, String, String, String, String, bool)>(
        "SELECT id, creator_name, latest_text, cover_path, link_url, up_uid, is_new FROM radar_feed WHERE up_uid NOT LIKE 'seed_%' ORDER BY published_at DESC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let items: Vec<Value> = rows.into_iter().map(|(id, name, text, cover, link, uid, is_new)| {
        let cover = crate::feed::proxied_cover(&format!("radar_{uid}"), &cover);
        json!({"id": id, "creatorName": name, "latestText": text, "cover": cover, "coverUrl": cover, "linkUrl": link, "isNew": is_new})
    }).collect();
    Json(json!({"items": items})).into_response()
}

async fn list_simple(state: &AppState, sql: &str) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (String, String, i32, i32, i32, String, String)>(
        sqlx::AssertSqlSafe(sql),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id, title, watched, total, latest, cover, link)| {
        let cover = crate::feed::proxied_cover(&id, &cover);
        json!({"id": id, "title": title, "watched": watched, "total": total, "latestEpisode": latest, "cover": cover, "coverUrl": cover, "linkUrl": link})
    }).collect())
}

async fn bili_videos(State(state): State<AppState>) -> Response {
    match crate::bili::owner_feed(&state).await {
        Ok(feed) => Json(feed).into_response(),
        Err(error) => fail(StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", &error),
    }
}

async fn acg_image(State(state): State<AppState>, Path(name): Path<String>) -> Response {
    let remote = match crate::bili::cover_source(&state, &name).await {
        Ok(Some(remote)) => remote,
        Ok(None) => return (StatusCode::NOT_FOUND, "not found").into_response(),
        Err(error) => return fail(StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", &error),
    };
    match crate::bili::fetch_cover(&state, &remote).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, sniffed_image_type(&bytes)),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=604800, immutable"),
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(()) => (StatusCode::BAD_GATEWAY, "cover unavailable").into_response(),
    }
}

pub(crate) fn sniffed_image_type(bytes: &[u8]) -> HeaderValue {
    let mime = if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "application/octet-stream"
    };
    HeaderValue::from_static(mime)
}

#[derive(Deserialize)]
struct WallpaperQuery {
    source: Option<String>,
    #[serde(rename = "apiOnly")]
    api_only: Option<String>,
}

fn wants_api_wallpaper(source: Option<&str>, api_only: Option<&str>) -> bool {
    source == Some("api") || api_only == Some("1")
}

#[derive(Deserialize)]
struct ViewBody {
    kind: String,
    id: String,
}

/// 浏览上报：ZINCRBY 周榜，不写 DB。未配 Redis 时静默 no-op。
async fn client_view(State(state): State<AppState>, AppJson(body): AppJson<ViewBody>) -> Response {
    let kind = match body.kind.as_str() {
        "article" | "journal" | "wallpaper" => body.kind.as_str(),
        _ => return fail(StatusCode::BAD_REQUEST, "INVALID_KIND", "浏览类型无效。"),
    };
    let id = body.id.trim();
    if id.is_empty() || id.len() > 64 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_ID", "浏览 ID 无效。");
    }
    let key = crate::cache::week_key(&format!("views:{kind}"));
    crate::cache::zincrby(&state, &key, id, 1.0).await;
    Json(json!({"ok": true})).into_response()
}

#[derive(Deserialize)]
struct HotQuery {
    kind: Option<String>,
}

/// 本周热门：周榜 Top6 关联标题/封面。未配 Redis 时 configured=false。
async fn client_hot(State(state): State<AppState>, Query(query): Query<HotQuery>) -> Response {
    let kind = match query.kind.as_deref() {
        Some("journal") => "journal",
        _ => "article",
    };
    if !state.cfg.redis_ready() {
        return Json(json!({"configured": false, "items": []})).into_response();
    }
    let key = crate::cache::week_key(&format!("views:{kind}"));
    let top = crate::cache::ztop(&state, &key, 6).await;
    let mut items = Vec::new();
    if let Some(pool) = state.db.as_ref() {
        if let Ok(entries) = crate::site_store::list(pool, kind).await {
            for (id, views) in top {
                if let Some(entry) = entries.iter().find(|e| e.id.to_string() == id) {
                    items.push(json!({
                        "id": entry.id,
                        "title": entry.title,
                        "views": views as i64,
                        "url": if kind == "journal" { format!("/build/post/{}", entry.id) } else { format!("/blog/{}", entry.id) },
                    }));
                }
            }
        }
    }
    Json(json!({"configured": true, "items": items})).into_response()
}

#[derive(Deserialize)]
struct ImageSearchQuery {
    q: String,
    limit: Option<u32>,
}

/// 生成图文字搜图：配了 Zilliz 走 BM25（type=image），否则回退 prompt LIKE 查库。
async fn client_image_search(
    State(state): State<AppState>,
    Query(query): Query<ImageSearchQuery>,
) -> Response {
    let q = query.q.trim();
    if q.is_empty() {
        return Json(json!({"configured": state.cfg.vector_ready(), "items": []})).into_response();
    }
    let limit = query.limit.unwrap_or(24).clamp(1, 60);
    if state.cfg.vector_ready() {
        let body = json!({
            "collectionName": state.cfg.vector_collection,
            "data": [q],
            "annsField": "sparse",
            "limit": limit,
            "filter": "type == \"image\"",
            "outputFields": ["title", "url", "excerpt", "type", "id"],
            "searchParams": {"metric_type": "BM25", "params": {}}
        });
        let sent = state
            .http
            .post(format!(
                "{}/v2/vectordb/entities/search",
                state.cfg.vector_uri.trim_end_matches('/')
            ))
            .bearer_auth(&state.cfg.vector_token)
            .timeout(Duration::from_secs(6))
            .json(&body)
            .send()
            .await;
        if let Ok(response) = sent {
            if response.status().is_success() {
                if let Ok(payload) = response.json::<Value>().await {
                    if let Ok(hits) = crate::search_hits::hits_from_milvus(&payload) {
                        let items: Vec<Value> = hits
                            .iter()
                            .map(|hit| {
                                json!({
                                    "imageUrl": hit.url,
                                    "prompt": hit.excerpt,
                                    "score": hit.score,
                                })
                            })
                            .collect();
                        return Json(json!({"configured": true, "items": items})).into_response();
                    }
                }
            }
        }
        // 向量查询失败时回退查库，保证搜索可用。
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"configured": false, "items": []})).into_response();
    };
    let pattern = format!("%{q}%");
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, prompt, image_url FROM ai_images WHERE prompt ILIKE $1 ORDER BY id DESC LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit as i64)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .iter()
                .map(|(id, prompt, url)| json!({"id": id.to_string(), "imageUrl": url, "prompt": prompt}))
                .collect();
            Json(json!({"configured": false, "items": items})).into_response()
        }
        Err(error) => query_failed(&error),
    }
}

async fn wallpaper_draw(
    State(state): State<AppState>,
    Query(query): Query<WallpaperQuery>,
) -> Response {
    let api_only = wants_api_wallpaper(query.source.as_deref(), query.api_only.as_deref());
    let fallback = if api_only {
        let seed = Utc::now().format("%Y%m%d%H%M%S");
        json!({
            "id": format!("external-api-placeholder-{seed}"),
            "url": format!("https://picsum.photos/seed/taozhiyy-{seed}/1920/1080"),
            "previewUrl": format!("https://picsum.photos/seed/taozhiyy-{seed}/960/540"),
            "source": "external-api-placeholder",
            "author": "Lorem Picsum",
            "sourceUrl": "https://picsum.photos/",
            "licenseNote": "图池里还没有接口壁纸",
            "kind": "api",
            "status": "active",
            "addedAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
    } else {
        json!({
            "id": "fallback-archive",
            "url": public_url(&state.cfg, "wallpapers/fallback.jpg"),
            "previewUrl": public_url(&state.cfg, "wallpapers/fallback.jpg"),
            "source": "r2",
            "author": "taozhiyy",
            "sourceUrl": "https://www.miku.coffee/",
            "licenseNote": "站点图池为空时的占位",
            "kind": "fallback",
            "status": "active",
            "addedAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
    };
    let Some(pool) = state.db.as_ref() else {
        return Json(if api_only {
            json!({"item": fallback, "fallback": true, "apiOnly": true})
        } else {
            json!({"item": fallback, "fallback": true})
        })
        .into_response();
    };
    let kind = if api_only { Some("api") } else { None };
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, DateTime<Utc>)>(
        "SELECT id, url, preview_url, source, author, source_url, license_note, kind, added_at FROM wallpapers WHERE status = 'active' AND ($1::text IS NULL OR kind = $1) ORDER BY random() LIMIT 1",
    )
    .bind(kind)
    .fetch_optional(pool)
    .await;
    let row = match row {
        Ok(row) => row,
        Err(error) => return query_failed(&error),
    };
    let row = if row.is_none() && api_only {
        match crate::wallpaper::sync_pool(&state).await {
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(error = %error, "wallpaper sync failed, serving fallback");
                return Json(json!({"item": fallback, "fallback": true, "apiOnly": true}))
                    .into_response();
            }
        }
        match sqlx::query_as::<_, (String, String, String, String, String, String, String, String, DateTime<Utc>)>(
            "SELECT id, url, preview_url, source, author, source_url, license_note, kind, added_at FROM wallpapers WHERE status = 'active' AND kind = 'api' ORDER BY random() LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        {
            Ok(value) => value,
            Err(error) => return query_failed(&error),
        }
    } else {
        row
    };
    let Some((id, url, preview, source, author, source_url, license, kind, added)) = row else {
        return Json(if api_only {
            json!({"item": fallback, "fallback": true, "apiOnly": true})
        } else {
            json!({"item": fallback, "fallback": true})
        })
        .into_response();
    };
    let item = json!({"id": id, "url": url, "previewUrl": preview, "source": source, "author": author, "sourceUrl": source_url, "licenseNote": license, "kind": kind, "status": "active", "addedAt": added.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)});
    let view_key = crate::cache::week_key("views:wallpaper");
    crate::cache::zincrby(&state, &view_key, &id, 1.0).await;
    Json(if api_only {
        json!({"item": item, "apiOnly": true})
    } else {
        json!({"item": item})
    })
    .into_response()
}

async fn search(State(state): State<AppState>, Query(query): Query<SearchQuery>) -> Response {
    if !state.cfg.vector_ready() {
        return Json(json!({"configured": false, "items": []})).into_response();
    }
    // limit 默认 10，上限 50，和迁移计划里的接口约定一致。
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let mut search_body = json!({
        "collectionName": state.cfg.vector_collection,
        "data": [query.q],
        "annsField": "sparse",
        "limit": limit,
        "outputFields": ["title", "url", "excerpt", "type", "id"],
        "searchParams": {"metric_type": "BM25", "params": {}}
    });
    // type 过滤走白名单，避免把用户输入拼进 filter 表达式。
    if let Some(kind) = query.kind.as_deref() {
        if matches!(kind, "blog" | "journal" | "image") {
            search_body["filter"] = json!(format!("type == \"{kind}\""));
        }
    }
    // Zilliz Cloud（Milvus REST v2）BM25 全文检索：data 直接传查询原文，
    // BM25 Function 把查询编成稀疏向量，metric_type 固定 BM25。
    let response = match state
        .http
        .post(format!(
            "{}/v2/vectordb/entities/search",
            state.cfg.vector_uri.trim_end_matches('/')
        ))
        .bearer_auth(&state.cfg.vector_token)
        .json(&search_body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::error!(error = %error, "vector query transport failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "SEARCH_UPSTREAM",
                "搜索服务暂时不可用，请稍后再试",
            );
        }
    };
    if !response.status().is_success() {
        return fail(
            StatusCode::BAD_GATEWAY,
            "SEARCH_UPSTREAM",
            &format!("HTTP {}", response.status()),
        );
    }
    let body = match response.json::<Value>().await {
        Ok(body) => body,
        Err(error) => {
            tracing::error!(error = %error, "vector query response parse failed");
            return fail(
                StatusCode::BAD_GATEWAY,
                "SEARCH_UPSTREAM",
                "搜索服务返回异常，请稍后再试",
            );
        }
    };
    match crate::search_hits::hits_from_milvus(&body) {
        Ok(items) => Json(json!({"configured": true, "items": items})).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "vector hits parse failed");
            fail(
                StatusCode::BAD_GATEWAY,
                "SEARCH_UPSTREAM",
                "搜索结果解析失败，请稍后再试",
            )
        }
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<u32>,
    /// 限定命中类型：blog / journal / image。空则全类型。
    #[serde(rename = "type")]
    kind: Option<String>,
}

async fn sync_trigger(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    // QStash 定时调度走签名分支；人工触发仍走令牌/站长会话。
    if headers.get("upstash-signature").is_some() {
        if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
            return *response;
        }
    } else if let Err(response) = authorize_sync_trigger(&state, &headers).await {
        return *response;
    }
    if state
        .sync_running
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .is_err()
    {
        return (
            StatusCode::ACCEPTED,
            Json(json!({"queued": "already_running"})),
        )
            .into_response();
    }
    let job = state.clone();
    tokio::spawn(async move {
        let _guard = SyncRunningGuard(&job.sync_running);
        run_queued_sync(&job).await;
    });
    Json(json!({"queued": "sync"})).into_response()
}

struct SyncRunningGuard<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for SyncRunningGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

/// 同一时刻只允许一个同步在跑（和 Go 的 syncRunning 互斥一致）。
fn try_begin_sync(state: &AppState) -> Option<SyncRunningGuard<'_>> {
    state
        .sync_running
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .ok()
        .map(|_| SyncRunningGuard(&state.sync_running))
}

async fn run_queued_sync(state: &AppState) {
    if let Err(error) = crate::bili::sync_owner(state).await {
        tracing::error!(%error, "queued video sync failed");
    }
    if let Err(error) = crate::feed::sync_bangumi(state).await {
        tracing::error!(%error, "queued bangumi sync failed");
    }
    if let Err(error) = crate::feed::sync_radar(state, Utc::now()).await {
        tracing::error!(%error, "queued radar sync failed");
    }
}

async fn authorize_sync_trigger(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), Box<Response>> {
    let provided = crate::sync_auth::provided_token(
        header_text(headers, "x-sync-trigger-token").as_deref(),
        header_text(headers, header::AUTHORIZATION.as_str()).as_deref(),
    );
    if crate::sync_auth::token_matches(&state.cfg.sync_trigger_token, &provided) {
        return Ok(());
    }
    let user = session_user(state, headers).await;
    // 站长分支和 Go 的 syncTriggerOwnerAuthorized / requireOwnerSession 一致：
    // 必须是站长而且会话 unlimited。
    if user
        .as_ref()
        .is_some_and(|item| item.is_owner && item.unlimited)
    {
        return Ok(());
    }
    if state.cfg.sync_trigger_token.is_empty() {
        return if user.is_some() {
            Err(Box::new(fail(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "需要站长权限。",
            )))
        } else {
            Err(Box::new(fail(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "请先登录。",
            )))
        };
    }
    Err(Box::new(fail(
        StatusCode::FORBIDDEN,
        "FORBIDDEN",
        "valid sync trigger token or owner session required",
    )))
}

async fn sync_bili(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    // 站长会话可以直接触发（和 require_owner 同一把尺子：is_owner 且 unlimited），
    // 否则必须有 QStash 签名。
    let owner = session_user(&state, &headers)
        .await
        .is_some_and(|user| user.is_owner && user.unlimited);
    if !owner {
        if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
            return *response;
        }
    }
    let Some(_sync) = try_begin_sync(&state) else {
        return (
            StatusCode::ACCEPTED,
            Json(json!({"queued": "already_running"})),
        )
            .into_response();
    };
    let count = match crate::bili::sync_owner(&state).await {
        Ok(count) => count,
        Err(message) => return fail(StatusCode::BAD_GATEWAY, "SYNC_FAILED", &message),
    };
    match crate::feed::sync_bangumi(&state).await {
        Ok(saved) => Json(json!({"ok": true, "started": true, "count": count, "bangumi": saved}))
            .into_response(),
        Err(error) => fail(
            StatusCode::BAD_GATEWAY,
            "SYNC_FAILED",
            &format!("投稿已保存 {count} 条，番剧同步失败: {error}"),
        ),
    }
}

async fn sync_wallpaper(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    let Some(_sync) = try_begin_sync(&state) else {
        return (
            StatusCode::ACCEPTED,
            Json(json!({"queued": "already_running"})),
        )
            .into_response();
    };
    match crate::wallpaper::sync_pool(&state).await {
        Ok(count) => Json(json!({"ok": true, "count": count})).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "sync trigger failed");
            fail(
                StatusCode::BAD_GATEWAY,
                "SYNC_FAILED",
                "同步任务失败，请稍后再试",
            )
        }
    }
}

async fn sync_radar(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    let Some(_sync) = try_begin_sync(&state) else {
        return (
            StatusCode::ACCEPTED,
            Json(json!({"queued": "already_running"})),
        )
            .into_response();
    };
    match crate::feed::sync_radar(&state, Utc::now()).await {
        Ok(report) => Json(json!({"ok": true, "saved": report.saved, "failed": report.failed}))
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "sync trigger failed");
            fail(
                StatusCode::BAD_GATEWAY,
                "SYNC_FAILED",
                "同步任务失败，请稍后再试",
            )
        }
    }
}

async fn chat_quota(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = session_user(&state, &headers).await;
    let (limit, used) = match quota_of(&state, &headers, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    Json(quota_json(&state, &user, limit, used).await).into_response()
}

#[derive(Deserialize)]
struct ChatPageContext {
    #[serde(rename = "pageUrl")]
    page_url: Option<String>,
    #[serde(rename = "pageTitle")]
    page_title: Option<String>,
    #[serde(rename = "pagePath")]
    page_path: Option<String>,
    language: Option<String>,
    #[serde(rename = "siteSection")]
    site_section: Option<String>,
    headings: Option<Vec<String>>,
    #[serde(rename = "visibleText")]
    visible_text: Option<String>,
}

#[derive(Deserialize)]
struct ChatBody {
    message: Option<String>,
    #[serde(rename = "pageUrl")]
    page_url: Option<String>,
    #[serde(rename = "pageTitle")]
    page_title: Option<String>,
    #[serde(rename = "pageContext")]
    page_context: Option<ChatPageContext>,
}

async fn chat_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ChatBody>,
) -> Response {
    if let Err(response) = admit(&state, &headers, &state.chat_gate, "chat", 20).await {
        return *response;
    }
    let user = session_user(&state, &headers).await;
    let (limit, used) = match quota_of(&state, &headers, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let raw_message = body.message.unwrap_or_default();
    let message = raw_message.trim();
    if message.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "EMPTY_MESSAGE",
            "请输入问题后再发送",
        );
    }
    if message.chars().count() > 2000 {
        return fail(
            StatusCode::BAD_REQUEST,
            "MESSAGE_TOO_LONG",
            "单次最多 2000 字哦",
        );
    }
    let mut quota = quota_json(&state, &user, limit, used).await;
    let answer = match fixed_answer(&state, message).await {
        Ok(answer) => answer,
        Err(response) => return *response,
    };
    if let Some((id, answer)) = answer {
        if let Err(response) = record_chat(&state, &user, ChatStat::Success).await {
            return *response;
        }
        quota["reply"] = json!(answer);
        quota["fixedAnswer"] = json!(true);
        quota["fixedAnswerId"] = json!(id);
        quota["chatEnabled"] = json!(true);
        quota["fixedAnswersEnabled"] = json!(true);
        return Json(quota).into_response();
    }
    if state.cfg.deepseek_api_key.is_empty() {
        if let Err(response) = record_chat(&state, &user, ChatStat::NotConfigured).await {
            return *response;
        }
        // Go 语义：没配 DeepSeek 时 chatEnabled = 是否存在固定答案，前端据此保留输入框。
        quota["error"] = json!("CHAT_NOT_CONFIGURED");
        quota["message"] = json!("小精灵还在沉睡中～站长配置 DeepSeek API Key 后就能聊天啦");
        return (StatusCode::SERVICE_UNAVAILABLE, Json(quota)).into_response();
    }
    if limit >= 0 && used >= limit {
        if let Err(response) = record_chat(&state, &user, ChatStat::QuotaDenied).await {
            return *response;
        }
        quota["error"] = json!("DAILY_LIMIT_EXCEEDED");
        quota["message"] = json!("今日提问次数用完啦，明天再来问我吧～");
        return (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response();
    }
    if let Err(response) = claim_chat_turn(&state, &headers, &user, limit).await {
        return *response;
    }
    let page = body.page_context.as_ref();
    let headings = page
        .and_then(|item| item.headings.as_ref())
        .map(|items| items.iter().map(String::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let prompt = crate::chat_prompt::user_content(
        message,
        body.page_url.as_deref().unwrap_or(""),
        body.page_title.as_deref().unwrap_or(""),
        crate::chat_prompt::PageFields {
            page_url: page.and_then(|item| item.page_url.as_deref()).unwrap_or(""),
            page_title: page
                .and_then(|item| item.page_title.as_deref())
                .unwrap_or(""),
            page_path: page
                .and_then(|item| item.page_path.as_deref())
                .unwrap_or(""),
            language: page.and_then(|item| item.language.as_deref()).unwrap_or(""),
            site_section: page
                .and_then(|item| item.site_section.as_deref())
                .unwrap_or(""),
            headings: &headings,
            visible_text: page
                .and_then(|item| item.visible_text.as_deref())
                .unwrap_or(""),
        },
    );
    // 模型走免费轮换（ai_model 每天随机挑），现检失败才用 DEEPSEEK_MODEL 兜底。
    // 免费模型经常被下线/限权（403/404/429）：标记坏模型换一个重试，别让一个模型卡死全天。
    let endpoint = format!(
        "{}/v1/chat/completions",
        state.cfg.deepseek_base_url.trim_end_matches('/')
    );
    let mut model = crate::ai_model::resolve(&state).await;
    let mut attempts = 0_u8;
    let response = loop {
        attempts += 1;
        let sent = state
            .http
            .post(&endpoint)
            .bearer_auth(&state.cfg.deepseek_api_key)
            .timeout(Duration::from_secs(45))
            .json(&crate::chat_prompt::request_body(&model, &prompt))
            .send()
            .await;
        let sent = match sent {
            Ok(item) => item,
            Err(error) => {
                release_chat_turn(&state, &headers, &user).await;
                if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                    return *response;
                }
                return chat_upstream(quota, &error.to_string());
            }
        };
        let status = sent.status();
        if status.is_success() {
            break sent;
        }
        let detail = sent.text().await.unwrap_or_default();
        // 403/404/429 是模型侧拒绝，换个模型再试；其它状态码（401/402/400…）重试也没用。
        let model_rejected = matches!(status.as_u16(), 403 | 404 | 429);
        if !model_rejected || attempts >= 3 {
            release_chat_turn(&state, &headers, &user).await;
            if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                return *response;
            }
            return chat_upstream(
                quota,
                &format!(
                    "HTTP {}: {}",
                    status,
                    detail.chars().take(200).collect::<String>()
                ),
            );
        }
        tracing::warn!(
            %model,
            %status,
            detail = %detail.chars().take(200).collect::<String>(),
            "chat model rejected; rotating to another free model"
        );
        crate::ai_model::mark_failed(&state, &model);
        model = crate::ai_model::resolve(&state).await;
    };
    let payload = match response.json::<Value>().await {
        Ok(body) => body,
        Err(error) => {
            release_chat_turn(&state, &headers, &user).await;
            if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                return *response;
            }
            return chat_upstream(quota, &error.to_string());
        }
    };
    // Go 只拿 trim 后的内容判空，回复本身原样返回。
    let Some(reply) = payload["choices"][0]["message"]["content"]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
    else {
        release_chat_turn(&state, &headers, &user).await;
        if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
            return *response;
        }
        return chat_upstream(quota, "模型没有返回回复");
    };
    if let Err(response) = record_chat(&state, &user, ChatStat::Success).await {
        return *response;
    }
    quota["used"] = json!(used + 1);
    quota["remaining"] = json!(if limit < 0 {
        -1
    } else {
        (limit - used - 1).max(0)
    });
    quota["reply"] = json!(reply);
    Json(quota).into_response()
}

async fn quota_json(state: &AppState, user: &Option<SessionUser>, limit: i64, used: i64) -> Value {
    // 对齐 Go chat.go 的标志位：chatEnabled = 配了 DeepSeek 或存在固定答案。
    let unlimited = user.as_ref().is_some_and(|item| item.unlimited);
    let deepseek = !state.cfg.deepseek_api_key.is_empty();
    let mut fixed = false;
    if let Some(pool) = state.db.as_ref() {
        fixed = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM ai_fixed_answers WHERE status = 'active')",
        )
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(%error, "chat fixed answers flag failed");
            false
        });
    }
    json!({
        "limit": if unlimited { -1 } else { limit },
        "used": used,
        "remaining": if unlimited { -1 } else { (limit - used).max(0) },
        "isLogin": user.is_some(),
        "unlimited": unlimited,
        "chatEnabled": deepseek || fixed,
        "deepseekEnabled": deepseek,
        "fixedAnswersEnabled": fixed,
        "imageEnabled": !state.cfg.dashscope_api_key.is_empty(),
    })
}

fn chat_daily_limit(user: &Option<SessionUser>) -> i64 {
    // 对齐 Go：只看会话的 unlimited（issue_session 给站长会话已置 unlimited=true）。
    if user.as_ref().is_some_and(|item| item.unlimited) {
        -1
    } else if user.is_some() {
        crate::chat_quota::USER_DAILY_LIMIT
    } else {
        crate::chat_quota::GUEST_DAILY_LIMIT
    }
}

fn chat_quota_key(user: &Option<SessionUser>, headers: &HeaderMap) -> String {
    if let Some(user) = user {
        format!("user:{}", user.id)
    } else {
        let agent = header_text(headers, header::USER_AGENT.as_str()).unwrap_or_default();
        crate::chat_quota::guest_key(&client_ip(headers), &agent)
    }
}

async fn quota_of(
    state: &AppState,
    headers: &HeaderMap,
    user: &Option<SessionUser>,
) -> Result<(i64, i64), Box<Response>> {
    let limit = chat_daily_limit(user);
    let key = format!(
        "quota:chat:{}:{}",
        crate::cache::sanitize(&chat_quota_key(user, headers)),
        quota_date()
    );
    if let Some(used) = crate::cache::get_int(state, &key).await {
        return Ok((limit, used.max(0)));
    }
    let Some(pool) = state.db.as_ref() else {
        return Ok((limit, 0));
    };
    let used: i64 = sqlx::query_scalar(
        "SELECT used::bigint FROM ai_chat_quota WHERE identity_key = $1 AND quota_date = $2",
    )
    .bind(chat_quota_key(user, headers))
    .bind(quota_date())
    .fetch_optional(pool)
    .await
    .map_err(|error| Box::new(query_failed(&error)))?
    .unwrap_or(0);
    Ok((limit, used))
}

async fn claim_chat_turn(
    state: &AppState,
    headers: &HeaderMap,
    user: &Option<SessionUser>,
    limit: i64,
) -> Result<(), Box<Response>> {
    let qkey = format!(
        "quota:chat:{}:{}",
        crate::cache::sanitize(&chat_quota_key(user, headers)),
        quota_date()
    );
    if state.cfg.redis_ready() {
        let pace_key = format!(
            "pace:chat:{}",
            crate::cache::sanitize(&chat_quota_key(user, headers))
        );
        match crate::cache::set_nx(state, &pace_key, Duration::from_secs(5)).await {
            Some(false) => {
                record_chat(state, user, ChatStat::RateDenied).await?;
                let used = crate::cache::get_int(state, &qkey).await.unwrap_or(0);
                let mut quota = quota_json(state, user, limit, used).await;
                quota["error"] = json!("RATE_LIMIT_EXCEEDED");
                quota["message"] = json!("提问太快啦，请稍等 5 秒再试～");
                return Err(Box::new(
                    (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response(),
                ));
            }
            Some(true) => {
                if let Some(used) =
                    crate::cache::incr(state, &qkey, Duration::from_secs(26 * 3600)).await
                {
                    if limit >= 0 && used > limit {
                        crate::cache::decr(state, &qkey).await;
                        record_chat(state, user, ChatStat::QuotaDenied).await?;
                        let mut quota = quota_json(state, user, limit, used - 1).await;
                        quota["error"] = json!("DAILY_LIMIT_EXCEEDED");
                        quota["message"] = json!("今日提问次数用完啦，明天再来问我吧～");
                        return Err(Box::new(
                            (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response(),
                        ));
                    }
                    return Ok(());
                }
            }
            None => {}
        }
    }
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    let claimed = sqlx::query_as::<_, (i64, bool, bool)>(
        "WITH existing AS (
            SELECT used, (last_request_at IS NULL OR last_request_at <= now() - interval '5 seconds') AS rate_ok
            FROM ai_chat_quota WHERE identity_key = $1 AND quota_date = $2
         ), updated AS (
            INSERT INTO ai_chat_quota (identity_key, quota_date, used, last_request_at)
            VALUES ($1, $2, 1, now())
            ON CONFLICT (identity_key, quota_date) DO UPDATE
            SET used = ai_chat_quota.used + 1, last_request_at = now()
            WHERE (ai_chat_quota.last_request_at IS NULL
                OR ai_chat_quota.last_request_at <= now() - interval '5 seconds')
              AND ($3::bigint < 0 OR ai_chat_quota.used < $3::bigint)
            RETURNING used
         )
         SELECT used::bigint, true AS claimed, true AS rate_ok FROM updated
         UNION ALL
         SELECT used::bigint, false AS claimed, rate_ok FROM existing
         WHERE NOT EXISTS (SELECT 1 FROM updated)",
    )
    .bind(chat_quota_key(user, headers))
    .bind(quota_date())
    .bind(limit)
    .fetch_one(pool)
    .await;
    let (used, open, rate_ok) = match claimed {
        Ok(row) => row,
        Err(error) => return Err(Box::new(query_failed(&error))),
    };
    if open {
        return Ok(());
    }
    // Go reserveQuota 先判 5 秒间隔再判每日限额，两条都触发时报 RATE_LIMIT_EXCEEDED。
    if rate_ok && limit >= 0 && used >= limit {
        record_chat(state, user, ChatStat::QuotaDenied).await?;
        let mut quota = quota_json(state, user, limit, used).await;
        quota["error"] = json!("DAILY_LIMIT_EXCEEDED");
        quota["message"] = json!("今日提问次数用完啦，明天再来问我吧～");
        return Err(Box::new(
            (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response(),
        ));
    }
    record_chat(state, user, ChatStat::RateDenied).await?;
    let mut quota = quota_json(state, user, limit, used).await;
    quota["error"] = json!("RATE_LIMIT_EXCEEDED");
    quota["message"] = json!("提问太快啦，请稍等 5 秒再试～");
    Err(Box::new(
        (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response(),
    ))
}

/// 配额日界按东八区算，和 Go 版的本地日界一致。
fn quota_date() -> chrono::NaiveDate {
    (Utc::now() + chrono::Duration::hours(8)).date_naive()
}

/// 模型失败后退回预占的次数。
async fn release_chat_turn(state: &AppState, headers: &HeaderMap, user: &Option<SessionUser>) {
    let qkey = format!(
        "quota:chat:{}:{}",
        crate::cache::sanitize(&chat_quota_key(user, headers)),
        quota_date()
    );
    if crate::cache::decr(state, &qkey).await.is_some() {
        return;
    }
    let Some(pool) = state.db.as_ref() else {
        return;
    };
    if let Err(error) = sqlx::query(
        "UPDATE ai_chat_quota SET used = GREATEST(used - 1, 0) WHERE identity_key = $1 AND quota_date = $2",
    )
    .bind(chat_quota_key(user, headers))
    .bind(quota_date())
    .execute(pool)
    .await
    {
        tracing::warn!(%error, "chat quota release failed");
    }
}

enum ChatStat {
    Success,
    UpstreamError,
    QuotaDenied,
    RateDenied,
    NotConfigured,
}

async fn record_chat(
    state: &AppState,
    user: &Option<SessionUser>,
    stat: ChatStat,
) -> Result<(), Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    let (success, upstream, quota, rate, missing) = match stat {
        ChatStat::Success => (1, 0, 0, 0, 0),
        ChatStat::UpstreamError => (0, 1, 0, 0, 0),
        ChatStat::QuotaDenied => (0, 0, 1, 0, 0),
        ChatStat::RateDenied => (0, 0, 0, 1, 0),
        ChatStat::NotConfigured => (0, 0, 0, 0, 1),
    };
    // Go 只在真实调用时区分游客/会员，not_configured 不计。
    let (guest, member) = if missing == 1 {
        (0, 0)
    } else {
        (i32::from(user.is_none()), i32::from(user.is_some()))
    };
    let hour = Utc::now().format("%Y-%m-%dT%H").to_string();
    let day = Utc::now().date_naive();
    let hourly = sqlx::query(
        "INSERT INTO ai_chat_hourly (bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (bucket) DO UPDATE SET success = ai_chat_hourly.success + EXCLUDED.success, upstream_error = ai_chat_hourly.upstream_error + EXCLUDED.upstream_error,
           quota_denied = ai_chat_hourly.quota_denied + EXCLUDED.quota_denied, rate_denied = ai_chat_hourly.rate_denied + EXCLUDED.rate_denied,
           not_configured = ai_chat_hourly.not_configured + EXCLUDED.not_configured, guest_calls = ai_chat_hourly.guest_calls + EXCLUDED.guest_calls, user_calls = ai_chat_hourly.user_calls + EXCLUDED.user_calls",
    )
    .bind(hour).bind(success).bind(upstream).bind(quota).bind(rate).bind(missing).bind(guest).bind(member)
    .execute(pool).await;
    if let Err(error) = hourly {
        tracing::warn!(%error, "chat hourly stat write failed");
    }
    let daily = sqlx::query(
        "INSERT INTO ai_chat_daily (bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (bucket) DO UPDATE SET success = ai_chat_daily.success + EXCLUDED.success, upstream_error = ai_chat_daily.upstream_error + EXCLUDED.upstream_error,
           quota_denied = ai_chat_daily.quota_denied + EXCLUDED.quota_denied, rate_denied = ai_chat_daily.rate_denied + EXCLUDED.rate_denied,
           not_configured = ai_chat_daily.not_configured + EXCLUDED.not_configured, guest_calls = ai_chat_daily.guest_calls + EXCLUDED.guest_calls, user_calls = ai_chat_daily.user_calls + EXCLUDED.user_calls",
    )
    .bind(day).bind(success).bind(upstream).bind(quota).bind(rate).bind(missing).bind(guest).bind(member)
    .execute(pool).await;
    if let Err(error) = daily {
        tracing::warn!(%error, "chat daily stat write failed");
    }
    Ok(())
}

async fn fixed_answer(
    state: &AppState,
    message: &str,
) -> Result<Option<(i64, String)>, Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(None);
    };
    let normalized = crate::fixed_answers::normalize_question(message);
    if normalized.is_empty() {
        return Ok(None);
    }
    sqlx::query_as(
        "SELECT id, answer FROM ai_fixed_answers WHERE status = 'active' AND normalized_question = $1 LIMIT 1",
    )
    .bind(normalized)
    .fetch_optional(pool)
    .await
    .map_err(|error| Box::new(query_failed(&error)))
}

fn chat_upstream(mut quota: Value, error: &str) -> Response {
    tracing::error!(error, "chat upstream failed");
    quota["error"] = json!("UPSTREAM_ERROR");
    quota["message"] = json!("小精灵暂时走神了，请稍后再试～");
    (StatusCode::SERVICE_UNAVAILABLE, Json(quota)).into_response()
}

#[derive(Deserialize)]
struct DaysQuery {
    days: Option<String>,
}

async fn chat_stats(State(state): State<AppState>, Query(query): Query<DaysQuery>) -> Response {
    let days = match query
        .days
        .as_deref()
        .and_then(|value| value.parse::<i64>().ok())
    {
        Some(n) if (1..=90).contains(&n) => n,
        _ => 14,
    };
    let today = Utc::now().date_naive();
    let since = today - chrono::Duration::days(days - 1);
    let (daily, hourly) = if let Some(pool) = state.db.as_ref() {
        let prefix = Utc::now().format("%Y-%m-%dT").to_string();
        let stats_limit = Duration::from_secs(2);
        let (daily, hourly, quota) = tokio::join!(
            tokio::time::timeout(
                stats_limit,
                sqlx::query_as::<_, (NaiveDate, i32, i32, i32, i32, i32, i32, i32)>(
                    "SELECT bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls FROM ai_chat_daily WHERE bucket >= $1 ORDER BY bucket",
                )
                .bind(since)
                .fetch_all(pool),
            ),
            tokio::time::timeout(
                stats_limit,
                sqlx::query_as::<_, (String, i32, i32, i32, i32, i32)>(
                    "SELECT bucket, success, upstream_error, quota_denied, rate_denied, not_configured FROM ai_chat_hourly WHERE bucket LIKE $1 || '%' ORDER BY bucket",
                )
                .bind(&prefix)
                .fetch_all(pool),
            ),
            tokio::time::timeout(
                stats_limit,
                sqlx::query_as::<_, (NaiveDate, i64, i64, i64)>(
                    "SELECT quota_date, COALESCE(SUM(used), 0)::bigint, COALESCE(SUM(CASE WHEN identity_key LIKE 'guest:%' THEN used ELSE 0 END), 0)::bigint, COALESCE(SUM(CASE WHEN identity_key LIKE 'user:%' THEN used ELSE 0 END), 0)::bigint FROM ai_chat_quota WHERE quota_date >= $1 GROUP BY quota_date",
                )
                .bind(since)
                .fetch_all(pool),
            ),
        );
        let mut daily = stats_rows(daily);
        let hourly = stats_rows(hourly);
        merge_quota_daily(&mut daily, &stats_rows(quota));
        (daily, hourly)
    } else {
        (Vec::new(), Vec::new())
    };
    Json(chat_stats_json(
        &crate::ai_model::cached_or_default(&state),
        !state.cfg.deepseek_api_key.is_empty(),
        today,
        &daily,
        &hourly,
    ))
    .into_response()
}

type DailyRow = (NaiveDate, i32, i32, i32, i32, i32, i32, i32);
type HourlyRow = (String, i32, i32, i32, i32, i32);

fn stats_rows<T>(
    result: Result<Result<Vec<T>, sqlx::Error>, tokio::time::error::Elapsed>,
) -> Vec<T> {
    match result {
        Ok(Ok(rows)) => rows,
        Ok(Err(error)) => {
            tracing::warn!(%error, "chat stats query failed");
            Vec::new()
        }
        Err(_) => {
            tracing::warn!("chat stats query timed out");
            Vec::new()
        }
    }
}

fn merge_quota_daily(daily: &mut Vec<DailyRow>, quota: &[(NaiveDate, i64, i64, i64)]) {
    for (date, used, guest, user) in quota {
        if *used <= 0 {
            continue;
        }
        let used = i32::try_from(*used).unwrap_or(i32::MAX);
        let guest = i32::try_from(*guest).unwrap_or(i32::MAX);
        let user = i32::try_from(*user).unwrap_or(i32::MAX);
        if let Some(row) = daily.iter_mut().find(|row| row.0 == *date) {
            if row.1 < used {
                row.1 = used;
            }
            if row.6 < guest {
                row.6 = guest;
            }
            if row.7 < user {
                row.7 = user;
            }
        } else {
            daily.push((*date, used, 0, 0, 0, 0, guest, user));
        }
    }
    daily.sort_by_key(|row| row.0);
}

fn chat_stats_json(
    model: &str,
    configured: bool,
    today: NaiveDate,
    daily: &[DailyRow],
    hourly: &[HourlyRow],
) -> Value {
    let daily_items: Vec<Value> = daily.iter().map(|(date, success, upstream, quota, rate, missing, guest, member)| {
        let total = success + upstream + quota + rate + missing;
        json!({"date": date.to_string(), "success": success, "upstreamError": upstream, "quotaDenied": quota, "rateDenied": rate, "notConfigured": missing, "guestCalls": guest, "userCalls": member, "total": total})
    }).collect();
    let mut by_hour = [0; 24].map(|_| (0, 0, 0, 0, 0));
    for (bucket, success, upstream, quota, rate, missing) in hourly {
        let hour = bucket
            .get(11..13)
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|hour| *hour < 24);
        if let Some(hour) = hour {
            let slot = &mut by_hour[hour];
            slot.0 += success;
            slot.1 += upstream;
            slot.2 += quota;
            slot.3 += rate;
            slot.4 += missing;
        }
    }
    let hourly_items: Vec<Value> = by_hour.iter().enumerate().map(|(hour, (success, upstream, quota, rate, missing))| {
        json!({"hour": format!("{hour:02}"), "success": success, "upstreamError": upstream, "quotaDenied": quota, "rateDenied": rate, "notConfigured": missing, "total": success + upstream + quota + rate})
    }).collect();
    let today_key = today.to_string();
    let mut today_success = 0;
    let mut today_total = 0;
    let mut period_success = 0;
    let mut period_total = 0;
    for item in &daily_items {
        let success = item["success"].as_i64().unwrap_or(0);
        let total = item["total"].as_i64().unwrap_or(0);
        period_success += success;
        period_total += total;
        if item["date"].as_str() == Some(today_key.as_str()) {
            today_success = success;
            today_total = total;
        }
    }
    if today_total == 0 {
        for item in &hourly_items {
            today_success += item["success"].as_i64().unwrap_or(0);
            today_total += item["total"].as_i64().unwrap_or(0);
        }
    }
    json!({
        "model": model,
        "configured": configured,
        "summary": {
            "todaySuccess": today_success,
            "todayTotal": today_total,
            "periodSuccess": period_success,
            "periodTotal": period_total,
            "successRate": if period_total > 0 { (period_success as f64) / (period_total as f64) } else { 0.0 },
            "successRateText": percent_text(period_success, period_total)
        },
        "daily": daily_items,
        "hourlyToday": hourly_items
    })
}

fn percent_text(success: i64, total: i64) -> String {
    if total <= 0 || success <= 0 {
        return "0%".into();
    }
    format!(
        "{}%",
        ((success as f64) / (total as f64) * 100.0 + 0.5) as i64
    )
}

async fn image_quota(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = session_user(&state, &headers).await;
    let (limit, used) = match image_quota_of(&state, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    Json(image_quota_json(&state, &user, limit, used)).into_response()
}

fn image_quota_json(state: &AppState, user: &Option<SessionUser>, limit: i64, used: i64) -> Value {
    let unlimited = user
        .as_ref()
        .is_some_and(|item| item.unlimited || item.is_owner);
    let enabled = crate::image_gen::configured(&state.cfg);
    json!({
        "limit": if unlimited { -1 } else { limit },
        "used": used,
        "remaining": if unlimited { -1 } else { (limit - used).max(0) },
        "isLogin": user.is_some(),
        "unlimited": unlimited,
        "imageEnabled": enabled,
        "canGenerate": user.is_some() && enabled && (unlimited || used < limit),
        "model": crate::image_gen::public_model(&state.cfg),
        "promptExtend": false,
        "userId": user.as_ref().map(|item| item.id).unwrap_or(0)
    })
}

async fn image_quota_of(
    state: &AppState,
    user: &Option<SessionUser>,
) -> Result<(i64, i64), Box<Response>> {
    const LIMIT: i64 = 3;
    let Some(user) = user else {
        return Ok((0, 0));
    };
    let key = format!("quota:image:user:{}:{}", user.id, quota_date());
    if let Some(used) = crate::cache::get_int(state, &key).await {
        return Ok((LIMIT, used.max(0)));
    }
    let Some(pool) = state.db.as_ref() else {
        return Ok((LIMIT, 0));
    };
    let used: i64 = sqlx::query_scalar(
        "SELECT used::bigint FROM ai_image_quota WHERE identity_key = $1 AND quota_date = $2",
    )
    .bind(format!("user:{}", user.id))
    .bind(quota_date())
    .fetch_optional(pool)
    .await
    .map_err(|error| Box::new(query_failed(&error)))?
    .unwrap_or(0);
    Ok((LIMIT, used))
}

/// 生成前原子预占一次额度；满额或 5 秒间隔不足时返回 429。
async fn claim_image_quota(
    state: &AppState,
    user: &SessionUser,
    limit: i64,
) -> Result<(), Box<Response>> {
    let qkey = format!("quota:image:user:{}:{}", user.id, quota_date());
    if state.cfg.redis_ready() {
        let pace_key = format!("pace:image:user:{}", user.id);
        match crate::cache::set_nx(state, &pace_key, Duration::from_secs(5)).await {
            Some(false) => {
                let used = crate::cache::get_int(state, &qkey).await.unwrap_or(0);
                let mut body = image_quota_json(state, &Some(user.clone()), limit, used);
                body["error"] = json!("RATE_LIMIT_EXCEEDED");
                body["message"] = json!("提问太快啦，请稍等 5 秒再试～");
                return Err(Box::new(
                    (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response(),
                ));
            }
            Some(true) => {
                if let Some(used) =
                    crate::cache::incr(state, &qkey, Duration::from_secs(26 * 3600)).await
                {
                    if limit >= 0 && used > limit {
                        crate::cache::decr(state, &qkey).await;
                        let mut body =
                            image_quota_json(state, &Some(user.clone()), limit, used - 1);
                        body["error"] = json!("DAILY_LIMIT_EXCEEDED");
                        body["message"] = json!("今日生图次数用完啦，明天再来画吧～");
                        return Err(Box::new(
                            (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response(),
                        ));
                    }
                    return Ok(());
                }
            }
            None => {}
        }
    }
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    let claimed = sqlx::query_as::<_, (i64, bool, bool)>(
        "WITH existing AS (
            SELECT used, (last_request_at IS NULL OR last_request_at <= now() - interval '5 seconds') AS rate_ok
            FROM ai_image_quota WHERE identity_key = $1 AND quota_date = $2
         ), updated AS (
            INSERT INTO ai_image_quota (identity_key, quota_date, used, last_request_at)
            VALUES ($1, $2, 1, now())
            ON CONFLICT (identity_key, quota_date) DO UPDATE
            SET used = ai_image_quota.used + 1, last_request_at = now()
            WHERE (ai_image_quota.last_request_at IS NULL
                OR ai_image_quota.last_request_at <= now() - interval '5 seconds')
              AND ($3::bigint < 0 OR ai_image_quota.used < $3::bigint)
            RETURNING used
         )
         SELECT used::bigint, true AS claimed, true AS rate_ok FROM updated
         UNION ALL
         SELECT used::bigint, false AS claimed, rate_ok FROM existing
         WHERE NOT EXISTS (SELECT 1 FROM updated)",
    )
    .bind(format!("user:{}", user.id))
    .bind(quota_date())
    .bind(limit)
    .fetch_one(pool)
    .await;
    let (used, open, rate_ok) = match claimed {
        Ok(row) => row,
        Err(error) => return Err(Box::new(query_failed(&error))),
    };
    if open {
        return Ok(());
    }
    // 和 Go reserveQuota 一致：先判 5 秒间隔再判每日限额，两条都触发时报 RATE_LIMIT_EXCEEDED。
    if rate_ok && limit >= 0 && used >= limit {
        let mut body = image_quota_json(state, &Some(user.clone()), limit, used);
        body["error"] = json!("DAILY_LIMIT_EXCEEDED");
        body["message"] = json!("今日生图次数用完啦，明天再来画吧～");
        return Err(Box::new(
            (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response(),
        ));
    }
    let mut body = image_quota_json(state, &Some(user.clone()), limit, used);
    body["error"] = json!("RATE_LIMIT_EXCEEDED");
    body["message"] = json!("提问太快啦，请稍等 5 秒再试～");
    Err(Box::new(
        (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response(),
    ))
}

/// 生成失败后退回预占的额度。
async fn release_image_quota(state: &AppState, user: &SessionUser) {
    let qkey = format!("quota:image:user:{}:{}", user.id, quota_date());
    if crate::cache::decr(state, &qkey).await.is_some() {
        return;
    }
    let Some(pool) = state.db.as_ref() else {
        return;
    };
    if let Err(error) = sqlx::query(
        "UPDATE ai_image_quota SET used = GREATEST(used - 1, 0) WHERE identity_key = $1 AND quota_date = $2",
    )
    .bind(format!("user:{}", user.id))
    .bind(quota_date())
    .execute(pool)
    .await
    {
        tracing::warn!(%error, "image quota release failed");
    }
}

#[derive(Deserialize)]
struct ImageBody {
    #[serde(default)]
    prompt: String,
    size: Option<String>,
}

async fn image_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ImageBody>,
) -> Response {
    if let Err(response) = admit(&state, &headers, &state.image_gate, "image", 6).await {
        return *response;
    }
    let user = session_user(&state, &headers).await;
    let (limit, used) = match image_quota_of(&state, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let quota = image_quota_json(&state, &user, limit, used);
    let Some(current) = user.as_ref() else {
        let mut body = quota;
        body["error"] = json!("LOGIN_REQUIRED");
        body["message"] = json!("请先登录后再生图，避免额度被刷爆。");
        return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
    };
    if !crate::image_gen::configured(&state.cfg) {
        let mut body = quota;
        body["error"] = json!("IMAGE_NOT_CONFIGURED");
        body["message"] = json!("生图还没配置 Agnes API Key，站长配好后就能用了。");
        return (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response();
    }
    // 和 Go 的 normalizeAIImagePrompt 一致：压空白后再判空、判长度。
    let prompt = crate::image_gen::normalize_prompt(&body.prompt);
    if prompt.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "EMPTY_PROMPT",
            "先写一句想生成什么画面吧。",
        );
    }
    if prompt.chars().count() > 1200 {
        return fail(
            StatusCode::BAD_REQUEST,
            "PROMPT_TOO_LONG",
            "生图描述最多 1200 个字。",
        );
    }
    let unlimited = current.unlimited || current.is_owner;
    if !unlimited && used >= limit {
        let mut body = quota;
        body["error"] = json!("DAILY_LIMIT_EXCEEDED");
        body["message"] = json!("今日生图次数用完啦，明天再来画吧～");
        return (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
    }
    // unlimited 也走预占（limit 传 -1 不封顶），和 Go 一样计 used 并受 5 秒间隔约束。
    if let Err(response) =
        claim_image_quota(&state, current, if unlimited { -1 } else { limit }).await
    {
        return *response;
    }
    let response =
        image_generate_flow(&state, &headers, &user, &prompt, &body, &quota, limit, used).await;
    if response.status() != StatusCode::OK {
        release_image_quota(&state, current).await;
    }
    response
}

#[allow(clippy::too_many_arguments)]
async fn image_generate_flow(
    state: &AppState,
    headers: &HeaderMap,
    user: &Option<SessionUser>,
    prompt: &str,
    request: &ImageBody,
    quota: &Value,
    limit: i64,
    used: i64,
) -> Response {
    let quota = quota.clone();
    let size = crate::image_gen::normalize_size(request.size.as_deref().unwrap_or(""));
    let generated = match crate::image_gen::generate(&state.http, &state.cfg, prompt, size).await {
        Ok(generated) => generated,
        Err(crate::image_gen::ImageGenError::PromptRejected) => {
            let mut body = quota;
            body["error"] = json!("IMAGE_PROMPT_REJECTED");
            body["message"] = json!(crate::image_gen::PROMPT_REJECTED_MESSAGE);
            return (StatusCode::BAD_REQUEST, Json(body)).into_response();
        }
        Err(crate::image_gen::ImageGenError::NotConfigured) => {
            let mut body = quota;
            body["error"] = json!("IMAGE_NOT_CONFIGURED");
            body["message"] = json!("生图还没配置 Agnes API Key，站长配好后就能用了。");
            return (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response();
        }
        Err(crate::image_gen::ImageGenError::Provider) => {
            let mut body = quota;
            body["error"] = json!("IMAGE_PROVIDER_ERROR");
            body["message"] = json!("Agnes 生图暂时失败了，请稍后再试。");
            return (StatusCode::BAD_GATEWAY, Json(body)).into_response();
        }
    };
    // Cloudflare 直接给图片字节，不用再下载；Agnes/百炼给临时地址。
    let stored = match &generated.bytes {
        Some(bytes) => crate::image_store::store_image_bytes(state, bytes).await,
        None => crate::image_store::store_remote_image(state, &generated.url).await,
    };
    let stored = match stored {
        Ok(stored) => stored,
        Err(error) => {
            let (status, code) = match error {
                crate::image_store::ImageStoreError::NotConfigured => {
                    (StatusCode::SERVICE_UNAVAILABLE, "R2_NOT_CONFIGURED")
                }
                crate::image_store::ImageStoreError::BadSource
                | crate::image_store::ImageStoreError::Download => {
                    (StatusCode::BAD_GATEWAY, "IMAGE_DOWNLOAD_ERROR")
                }
                crate::image_store::ImageStoreError::Upload => {
                    (StatusCode::BAD_GATEWAY, "R2_UPLOAD_ERROR")
                }
            };
            tracing::error!(error = %error, "image store failed");
            let message = match code {
                "R2_NOT_CONFIGURED" => "图床还没有配置好",
                "IMAGE_DOWNLOAD_ERROR" => "生成图片下载失败，请稍后再试",
                _ => "图片保存失败，请稍后再试",
            };
            // 和 Go 一致：失败响应也带上配额快照字段。
            let mut body = quota;
            body["error"] = json!(code);
            body["message"] = json!(message);
            return (status, Json(body)).into_response();
        }
    };
    let Some(pool) = state.db.as_ref() else {
        let mut body = quota;
        body["error"] = json!("DB_UNAVAILABLE");
        body["message"] = json!("生成图已保存到 R2，但数据库还没有连上");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();
    };
    // author 存邮箱作联表键（users.email 不可变更），画廊再 LEFT JOIN users 现算显示名。
    let author = user.as_ref().map(|item| item.email.as_str()).unwrap_or("");
    let inserted = sqlx::query_scalar::<_, i64>(
        "INSERT INTO ai_images (prompt, image_url, size, author, object_key) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(prompt)
    .bind(&stored.url)
    .bind(size)
    .bind(author)
    .bind(&stored.key)
    .fetch_one(pool)
    .await;
    let image_id = match inserted {
        Ok(id) => id,
        Err(error) => {
            // 原始错误只进日志，不回给客户端。
            tracing::error!(error = %error, "image record insert failed");
            let mut body = quota;
            body["error"] = json!("SERVER_ERROR");
            body["message"] = json!("生成图已保存到 R2，记录没有写上");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();
        }
    };
    // 异步把 prompt 写进向量索引，供广场文字搜图；未配 Zilliz 时静默跳过。
    {
        let s = state.clone();
        let p = prompt.to_string();
        let u = stored.url.clone();
        let a = author.to_string();
        tokio::spawn(async move {
            if let Err(error) = crate::jobs::index_image(&s, image_id, &p, &u, &a).await {
                tracing::warn!(%error, "image index failed");
            }
        });
    }
    if let Err(response) = note_audit(
        state,
        headers,
        AuditNote {
            event: "ai.image",
            outcome: "success",
            actor: user.as_ref().map(|item| item.id).unwrap_or(0),
            target_type: "ai_image",
            target_id: &stored.key,
            detail: "stored",
        },
    )
    .await
    {
        return *response;
    }
    // 预占已把 used 加一，成功响应直接展示预占后的口径。
    let mut out = image_quota_json(state, user, limit, used + 1);
    out["image"] = json!({"url": stored.url, "path": stored.key, "mimeType": stored.mime, "size": stored.size, "width": generated.width, "height": generated.height});
    out["size"] = json!(generated.size);
    out["model"] = json!(generated.model);
    out["requestId"] = json!(generated.request_id);
    out["promptExtend"] = json!(false);
    Json(out).into_response()
}

#[derive(Deserialize)]
struct GalleryQuery {
    limit: Option<String>,
    before: Option<String>,
}

async fn image_gallery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GalleryQuery>,
) -> Response {
    if let Err(response) = admit(&state, &headers, &state.gallery_gate, "gallery", 60).await {
        return *response;
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": [], "nextCursor": null})).into_response();
    };
    // 和 Go 一致地宽容解析：limit 不是数字就用默认值，不报 400。
    let limit = match query
        .limit
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => match value.parse::<i64>() {
            Ok(count) => count.clamp(1, 200),
            Err(_) => 60,
        },
        None => 60,
    };
    // before 解析不了时和 Go 一样只给空页，不回第一页。
    let before = match query
        .before
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        None => None,
        Some(value) => match DateTime::parse_from_rfc3339(value) {
            Ok(parsed) => Some(parsed.with_timezone(&Utc)),
            Err(_) => return Json(json!({"items": [], "nextCursor": null})).into_response(),
        },
    };
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            DateTime<Utc>,
            String,
            String,
            String,
        ),
    >(
        "SELECT g.prompt, g.image_url, g.size, g.created_at,
                COALESCE(u.email, ''), COALESCE(u.display_name, ''), g.author
         FROM ai_images g
         LEFT JOIN users u ON u.email = g.author
         WHERE ($1::timestamptz IS NULL OR g.created_at < $1)
         ORDER BY g.created_at DESC LIMIT $2",
    )
    .bind(before)
    .bind(limit)
    .fetch_all(pool)
    .await;
    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let full = rows.len() as i64 == limit;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(prompt, url, size, created, email, display_name, stored_author)| {
            // 和 Go 一样联表 users 现算 displayNameOrEmail：显示名空时用邮箱前缀，截 12 字。
            let author = if email.is_empty() && display_name.is_empty() {
                if stored_author.contains('@') {
                    // 新行存邮箱，用户已注销时和 Go 一样显示「访客」。
                    "访客".to_string()
                } else {
                    // 旧行是插入时的显示名快照，兜底直接展示。
                    crate::account::display_name_or_email("", &stored_author)
                }
            } else {
                crate::account::display_name_or_email(&email, &display_name)
            };
            json!({"prompt": prompt, "imageUrl": url, "size": size, "author": author, "createdAt": created.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)})
        })
        .collect();
    let next = if full {
        items.last().and_then(|item| item.get("createdAt").cloned())
    } else {
        None
    };
    Json(json!({"items": items, "nextCursor": next})).into_response()
}

async fn server_info(State(state): State<AppState>) -> Json<Value> {
    let host = match tokio::task::spawn_blocking(crate::host::sample).await {
        Ok(host) => host,
        Err(_) => crate::host::fallback(),
    };
    Json(json!({
        "vendor": env_label("SERVER_VENDOR", "UCloud 香港"),
        "region": env_label("SERVER_REGION", "亚太 · 香港"),
        "os": host.os,
        "arch": std::env::consts::ARCH,
        "goVersion": "rust",
        "cpuCores": host.cpu_cores,
        // Go 语义是活跃 goroutine 数，Rust 没有对应计数，用 tokio 工作线程规模近似。
        "goroutines": host.cpu_cores,
        "memory": {"usedMB": host.memory_used_mb, "totalMB": host.memory_total_mb, "usedRatio": host.memory_ratio, "source": host.memory_source},
        "cpuPercent": host.cpu_percent,
        "uptime": format_uptime(state.started.elapsed()),
        "uptimeSecs": state.started.elapsed().as_secs(),
        "status": "online",
        "serverTime": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }))
}

fn env_label(key: &str, fallback: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn format_uptime(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        return format!("{secs} 秒");
    }
    if secs < 3600 {
        return format!("{} 分钟", secs / 60);
    }
    if secs < 86_400 {
        return format!("{} 小时 {} 分", secs / 3600, (secs % 3600) / 60);
    }
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3600;
    if hours == 0 {
        format!("{days} 天")
    } else {
        format!("{days} 天 {hours} 小时")
    }
}

async fn client_network(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let ip = client_ip(&headers);
    // Go 这个接口永远 200：查地区失败只降级成空地区（label 回退「访客」），不报错。
    let region = match crate::guestbook::lookup_region(state.db.as_ref(), &state.http, &ip).await {
        Ok(region) => region,
        Err(error) => {
            tracing::warn!(%error, "client region lookup failed");
            String::new()
        }
    };
    Json(json!({
        "regionLabel": crate::guestbook::compact_region_label(&region),
        "ipRegion": region,
        "ipMasked": crate::guestbook::mask_ip(&ip),
        "serverTime": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }))
    .into_response()
}

pub(crate) fn client_ip(headers: &HeaderMap) -> String {
    if let Some(forwarded) = header_text(headers, "x-forwarded-for") {
        if let Some(first) = forwarded
            .split(',')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return first.to_string();
        }
    }
    header_text(headers, "x-real-ip").unwrap_or_else(|| "127.0.0.1".into())
}

fn header_text(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

async fn require_owner(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionUser, Box<Response>> {
    match session_user(state, headers).await {
        Some(user) if user.is_owner && user.unlimited => Ok(user),
        Some(user) if user.is_owner => Err(Box::new(fail(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "需要站长权限。",
        ))),
        Some(_) => Err(Box::new(fail(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "需要站长权限。",
        ))),
        None => Err(Box::new(fail(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "请先登录。",
        ))),
    }
}

async fn owner_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let pool = state.db.as_ref();
    let (user_total, registered_total) = if let Some(pool) = pool {
        let total: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
        {
            Ok(total) => total,
            Err(error) => return query_failed(&error),
        };
        let registered: i64 =
            match sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_owner = false")
                .fetch_one(pool)
                .await
            {
                Ok(total) => total,
                Err(error) => return query_failed(&error),
            };
        (total, registered)
    } else {
        (0, 0)
    };
    let drafts = match owner_draft_items(pool).await {
        Ok(items) => items,
        Err(error) => return query_failed(&error),
    };
    let answers = match fixed_answer_items(pool).await {
        Ok(items) => items,
        Err(error) => return query_failed(&error),
    };
    let notifications = match notification_items(pool).await {
        Ok(items) => items,
        Err(error) => return query_failed(&error),
    };
    let today_success: i64 = if let Some(pool) = pool {
        let prefix = Utc::now().format("%Y-%m-%dT").to_string();
        match sqlx::query_scalar(
            "SELECT COALESCE(SUM(success), 0) FROM ai_chat_hourly WHERE bucket LIKE $1 || '%'",
        )
        .bind(prefix)
        .fetch_one(pool)
        .await
        {
            Ok(value) => value,
            Err(error) => return query_failed(&error),
        }
    } else {
        0
    };
    Json(json!({
        "owner": {"id": user.id, "email": user.email, "displayName": crate::account::display_name_or_email(&user.email, &user.display_name)},
        "users": {"total": user_total, "registeredTotal": registered_total},
        "notifications": {"total": notifications.len(), "items": notifications},
        "ai": {"today": today_success, "fixedAnswers": answers},
        "uploads": {"maxBytes": 8 * 1024 * 1024, "baseURL": "/api/owner/uploads/"},
        "drafts": {"total": drafts.len(), "items": drafts}
    }))
    .into_response()
}

async fn owner_emails(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"registeredUsers": [], "guestbookContacts": []})).into_response();
    };
    let users = sqlx::query_as::<_, (i64, String, String, DateTime<Utc>)>(
        "SELECT id, email, display_name, created_at FROM users WHERE is_owner = false ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await;
    let users = match users {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let contacts = match sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>)>(
        "SELECT gm.id, gm.channel, gm.nickname, gm.content, gm.contact_email, COALESCE(u.email, ''), gm.created_at FROM guestbook_messages gm LEFT JOIN users u ON u.id = gm.user_id WHERE gm.status = 'visible' AND (btrim(gm.contact_email) <> '' OR COALESCE(btrim(u.email), '') <> '') ORDER BY gm.id DESC LIMIT 200",
    ).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    Json(json!({
        "registeredUsers": users.into_iter().map(|(id, email, name, created)| {
            json!({"id": id, "email": email, "displayName": crate::account::display_name_or_email(&email, &name), "createdAt": go_timestamp(created)})
        }).collect::<Vec<_>>(),
        "guestbookContacts": contacts.into_iter().filter_map(|(id, channel, nickname, content, submitted, account, created)| {
            let submitted = submitted.trim().to_lowercase();
            let account = account.trim().to_lowercase();
            let contact = if submitted.is_empty() { account.clone() } else { submitted };
            if contact.is_empty() {
                return None;
            }
            Some(json!({"id": id, "source": channel, "nickname": nickname, "content": content, "contactEmail": contact, "accountEmail": account, "createdAt": go_timestamp(created)}))
        }).collect::<Vec<_>>()
    })).into_response()
}

async fn owner_drafts(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let items = match owner_draft_items(state.db.as_ref()).await {
        Ok(items) => items,
        Err(error) => return query_failed(&error),
    };
    Json(json!({"items": items, "total": items.len()})).into_response()
}

async fn owner_draft_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<Value>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let title = body
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if title.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_TITLE",
            "草稿标题不能为空。",
        );
    }
    if title.chars().count() > 120 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_TITLE", "草稿标题太长。");
    }
    let kind = body
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let kind = if kind.is_empty() { "article" } else { kind };
    let text = body.get("body").and_then(Value::as_str).unwrap_or("");
    if text.len() > 200_000 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_BODY", "草稿正文太大。");
    }
    let cover = body
        .get("coverUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let status = if status.is_empty() { "draft" } else { status };
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>, DateTime<Utc>)>("INSERT INTO owner_drafts (kind, title, body, cover_url, status) VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at, updated_at")
        .bind(kind).bind(title).bind(text).bind(cover).bind(status).fetch_one(pool).await;
    let Ok((id, created, updated)) = row else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "草稿没有保存",
        );
    };
    let _owner = owner;
    Json(json!({"ok": true, "item": {"id": id, "kind": kind, "title": title, "body": text, "coverUrl": cover, "status": status, "createdAt": go_timestamp(created), "updatedAt": go_timestamp(updated)}})).into_response()
}

fn owner_payload(user: &SessionUser, item: Value) -> Value {
    json!({"ok": true, "item": item, "owner": {"email": user.email, "displayName": crate::account::display_name_or_email(&user.email, &user.display_name)}})
}

fn content_error(error: crate::site_store::StoreError, code: &str) -> Response {
    match error {
        crate::site_store::StoreError::Invalid(message) => {
            fail(StatusCode::BAD_REQUEST, code, message)
        }
        crate::site_store::StoreError::Database(message) => fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTENT_WRITE_FAILED",
            &message,
        ),
    }
}

/// 内容写路径统一失效：该栏目列表 + 最近列表的共享缓存。
async fn invalidate_content_caches(state: &AppState, kind: &str) {
    let kind_key = format!("content:{kind}");
    crate::cache::del(state, &[kind_key.as_str(), "content:recent"]).await;
}

/// 发布异步化：配了 QStash 就入队（带重试），否则进程内 spawn 尽力交付。
/// 两条路径都不阻塞发布响应；serverless 上 spawn 可能被冻结回收，QStash 才是可靠路径。
async fn enqueue_or_spawn(
    state: &AppState,
    path: &str,
    payload: Value,
    work: impl std::future::Future<Output = ()> + Send + 'static,
) {
    match crate::qstash::enqueue(
        &state.http,
        &state.cfg.qstash_token,
        &state.cfg.public_origin,
        path,
        &payload,
    )
    .await
    {
        Ok(()) => {}
        Err(error) => {
            tracing::warn!(%error, path, "qstash enqueue failed; falling back to in-process spawn");
            tokio::spawn(work);
        }
    }
}

fn owned_repo(state: &AppState) -> Option<(String, String, String)> {
    let branch = if state.cfg.github_branch.is_empty() {
        "main"
    } else {
        state.cfg.github_branch.as_str()
    };
    crate::github_publish::repo_from_config(&state.cfg.github_token, &state.cfg.github_repo, branch)
        .ok()
        .map(|(o, r, b)| (o.to_string(), r.to_string(), b.to_string()))
}

async fn run_publish_job(state: &AppState, payload: &Value) {
    let Some((owner, repo, branch)) = owned_repo(state) else {
        tracing::warn!("publish job skipped: github not configured");
        return;
    };
    let spec = crate::github_publish::RepoSpec {
        api_base: "https://api.github.com",
        token: &state.cfg.github_token,
        owner: &owner,
        repo: &repo,
        branch: &branch,
    };
    let kind = payload["kind"].as_str().unwrap_or("");
    let get = |key: &str| payload[key].as_str().unwrap_or("");
    let result = match kind {
        "article" => crate::github_publish::publish_article(
            &state.http,
            &spec,
            get("title"),
            get("markdown"),
            get("path"),
            Utc::now(),
        )
        .await
        .map(|_| ()),
        "friend" => crate::github_publish::publish_friend_direct(
            &state.http,
            &spec,
            crate::js_source::FriendInput {
                name: get("name"),
                desc: get("desc"),
                url: get("url"),
                avatar: get("avatar"),
            },
            Utc::now(),
        )
        .await
        .map(|_| ()),
        "moment" => crate::github_publish::publish_moment(
            &state.http,
            &spec,
            crate::js_source::MomentInput {
                year: get("year"),
                date: get("date"),
                kind: get("type"),
                category: get("category"),
                content: get("content"),
            },
            Utc::now(),
        )
        .await
        .map(|_| ()),
        "gallery" => crate::github_publish::publish_gallery(
            &state.http,
            &spec,
            crate::js_source::GalleryInput {
                album_id: get("albumId"),
                album_title: get("albumTitle"),
                image_url: get("imageUrl"),
            },
            Utc::now(),
        )
        .await
        .map(|_| ()),
        _ => {
            tracing::warn!(kind, "unknown publish job kind");
            return;
        }
    };
    if let Err(error) = result {
        tracing::warn!(%error, kind, "publish write-back failed");
    }
}

async fn internal_publish(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_JSON",
            "任务内容格式不正确。",
        );
    };
    run_publish_job(&state, &payload).await;
    Json(json!({"ok": true})).into_response()
}

async fn run_mail_job(state: &AppState, payload: &Value) {
    let get = |key: &str| payload[key].as_str().unwrap_or("");
    let notice = crate::mail::GuestbookNotice {
        channel: get("channel"),
        nickname: get("nickname"),
        content: get("content"),
        created_at: get("createdAt"),
        reply: payload["reply"].as_bool().unwrap_or(false),
    };
    if let Err(error) = crate::mail::notify_owner(&state.cfg, &notice).await {
        tracing::warn!(%error, "async owner notification failed");
    }
    let parent_id = payload["parentId"].as_i64().unwrap_or(0);
    if payload["adminReply"].as_bool().unwrap_or(false) && parent_id > 0 {
        if let Some(pool) = state.db.as_ref() {
            if let Err(error) =
                crate::mail::notify_reply(&state.cfg, pool, parent_id, &notice).await
            {
                tracing::warn!(%error, "async reply notification failed");
            }
        }
    }
}

async fn internal_mail(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_JSON",
            "任务内容格式不正确。",
        );
    };
    run_mail_job(&state, &payload).await;
    Json(json!({"ok": true})).into_response()
}

async fn internal_reindex(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    let images = match crate::jobs::index_images(&state).await {
        Ok(count) => count,
        Err(error) => {
            tracing::warn!(%error, "image reindex failed");
            0
        }
    };
    match crate::jobs::index_content(&state).await {
        Ok(count) => Json(json!({"ok": true, "indexed": count, "images": images})).into_response(),
        Err(error) => fail(StatusCode::BAD_GATEWAY, "INDEX_FAILED", &error.to_string()),
    }
}

/// 发布后的异步事：GitHub 写回 + 向量索引重建（邮件在留言侧单独入队）。
async fn dispatch_publish_extras(state: &AppState, payload: Value) {
    let state2 = state.clone();
    let payload2 = payload.clone();
    enqueue_or_spawn(state, "/api/internal/publish", payload, async move {
        run_publish_job(&state2, &payload2).await;
    })
    .await;
    let state3 = state.clone();
    enqueue_or_spawn(state, "/api/internal/reindex", json!({}), async move {
        if let Err(error) = crate::jobs::index_content(&state3).await {
            tracing::warn!(%error, "async reindex failed");
        }
    })
    .await;
}

async fn save_entry(
    state: &AppState,
    headers: &HeaderMap,
    owner: &SessionUser,
    event: &str,
    code: &str,
    entry: crate::site_store::NewEntry,
    draft_id: i64,
) -> Response {
    let pool = match db_required(state) {
        Ok(pool) => pool,
        Err(response) => return *response,
    };
    let kind = entry.kind.clone();
    let saved = match crate::site_store::insert(pool, entry).await {
        Ok(saved) => saved,
        Err(error) => return content_error(error, code),
    };
    invalidate_content_caches(state, &kind).await;
    if draft_id > 0 {
        if let Err(response) = mark_draft_published(state, draft_id).await {
            return *response;
        }
    }
    let target = saved.id.to_string();
    if let Err(response) = note_audit(
        state,
        headers,
        AuditNote {
            event,
            outcome: "success",
            actor: owner.id,
            target_type: "site_entry",
            target_id: &target,
            detail: "saved",
        },
    )
    .await
    {
        return *response;
    }
    let mut item = crate::site_store::entry_json(&saved);
    if let Some(object) = item.as_object_mut() {
        object.insert(
            "path".into(),
            Value::String(format!("site_entries/{}", saved.id)),
        );
    }
    Json(owner_payload(owner, item)).into_response()
}

async fn remove_entry(
    state: &AppState,
    headers: &HeaderMap,
    event: &str,
    code: &str,
    id: i64,
    kind: &str,
) -> Response {
    let owner = match require_owner(state, headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let pool = match db_required(state) {
        Ok(pool) => pool,
        Err(response) => return *response,
    };
    match crate::site_store::delete(pool, id, kind).await {
        Ok(true) => {
            let target = id.to_string();
            if let Err(response) = note_audit(
                state,
                headers,
                AuditNote {
                    event,
                    outcome: "success",
                    actor: owner.id,
                    target_type: "site_entry",
                    target_id: &target,
                    detail: "deleted",
                },
            )
            .await
            {
                return *response;
            }
            invalidate_content_caches(state, kind).await;
            Json(owner_payload(&owner, json!({"id": id, "kind": kind}))).into_response()
        }
        Ok(false) => fail(StatusCode::NOT_FOUND, code, "没有这条内容"),
        Err(error) => content_error(error, code),
    }
}

async fn content_recent(State(state): State<AppState>) -> Response {
    if let Some((body, false)) = crate::cache::get(&state, "content:recent").await {
        return Json(body).into_response();
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    match crate::site_store::list_recent(pool, 30).await {
        Ok(items) => {
            let items: Vec<Value> = items.iter().map(crate::site_store::entry_json).collect();
            let body = json!({"items": items});
            crate::cache::set(
                &state,
                "content:recent",
                &body,
                false,
                Duration::from_secs(30),
            )
            .await;
            Json(body).into_response()
        }
        Err(error) => content_error(error, "CONTENT_READ_FAILED"),
    }
}

async fn content_list(State(state): State<AppState>, Path(kind): Path<String>) -> Response {
    if !crate::site_store::known_kind(&kind) {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "栏目无效");
    }
    let key = format!("content:{kind}");
    if let Some((body, false)) = crate::cache::get(&state, &key).await {
        return Json(body).into_response();
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    match crate::site_store::list(pool, &kind).await {
        Ok(items) => {
            let items: Vec<Value> = items.iter().map(crate::site_store::entry_json).collect();
            let body = json!({"items": items});
            crate::cache::set(&state, &key, &body, false, Duration::from_secs(30)).await;
            Json(body).into_response()
        }
        Err(error) => content_error(error, "CONTENT_READ_FAILED"),
    }
}

async fn owner_publish(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<Value>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let title = body
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let markdown_body = body.get("body").and_then(Value::as_str).unwrap_or("");
    let cover = body.get("coverUrl").and_then(Value::as_str).unwrap_or("");
    let draft_id = body.get("draftId").and_then(Value::as_i64).unwrap_or(0);
    let now = Utc::now();
    let article = match crate::post_markdown::build_article(
        crate::post_markdown::ArticleInput {
            title,
            body: markdown_body,
            cover_url: cover,
        },
        now,
    ) {
        Ok(article) => article,
        Err(error) => {
            let message = error.to_string();
            let code = if message.starts_with("发布标题") {
                "INVALID_TITLE"
            } else if message.starts_with("发布正文") {
                "INVALID_BODY"
            } else {
                "INVALID_PUBLISH"
            };
            return fail(StatusCode::BAD_REQUEST, code, &message);
        }
    };
    let response = save_entry(
        &state,
        &headers,
        &owner,
        "owner.publish",
        "INVALID_PUBLISH",
        crate::site_store::NewEntry {
            kind: "article".into(),
            title: title.to_string(),
            body: markdown_body.trim().to_string(),
            image_url: cover.trim().to_string(),
            extra: json!({}),
        },
        draft_id,
    )
    .await;
    if response.status().is_success() {
        dispatch_publish_extras(
            &state,
            json!({"kind": "article", "title": title, "markdown": article.markdown, "path": article.path}),
        )
        .await;
    }
    response
}

async fn mark_draft_published(state: &AppState, draft_id: i64) -> Result<(), Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DRAFT_UPDATE_FAILED",
            "文章已写入仓库，但数据库还没有连上，草稿状态没有更新",
        )));
    };
    let updated = sqlx::query(
        "UPDATE owner_drafts SET status = 'published', updated_at = now() WHERE id = $1",
    )
    .bind(draft_id)
    .execute(pool)
    .await
    .map_err(|error| {
        Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DRAFT_UPDATE_FAILED",
            &format!("文章已写入仓库，草稿状态没有更新: {error}"),
        ))
    })?;
    if updated.rows_affected() == 0 {
        tracing::warn!(draft_id, "draft not found after publish; ignoring");
    }
    Ok(())
}

#[derive(Deserialize)]
struct FriendBody {
    #[serde(default)]
    name: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    avatar: String,
}

#[derive(Deserialize)]
struct MomentBody {
    #[serde(default)]
    year: String,
    #[serde(default)]
    date: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct GalleryBody {
    #[serde(rename = "albumId", default)]
    album_id: String,
    #[serde(rename = "albumTitle", default)]
    album_title: String,
    #[serde(rename = "imageUrl", default)]
    image_url: String,
}

fn publish_spec(state: &AppState) -> Result<crate::github_publish::RepoSpec<'_>, Box<Response>> {
    match crate::github_publish::repo_from_config(
        &state.cfg.github_token,
        &state.cfg.github_repo,
        &state.cfg.github_branch,
    ) {
        Ok((owner, repo, branch)) => Ok(crate::github_publish::RepoSpec {
            api_base: "https://api.github.com",
            token: &state.cfg.github_token,
            owner,
            repo,
            branch,
        }),
        Err(error) => Err(Box::new(fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "PUBLISH_NOT_CONFIGURED",
            &error.to_string(),
        ))),
    }
}

async fn publish_not_configured(
    state: &AppState,
    headers: &HeaderMap,
    actor: i64,
    event: &str,
    target: &str,
    response: Box<Response>,
) -> Response {
    if let Err(audit) = note_audit(
        state,
        headers,
        AuditNote {
            event,
            outcome: "failure",
            actor,
            target_type: "owner_publish",
            target_id: target,
            detail: "not_configured",
        },
    )
    .await
    {
        return *audit;
    }
    *response
}

struct PublishFailure<'a> {
    actor: i64,
    event: &'a str,
    target: &'a str,
    invalid_code: &'a str,
    github_code: &'a str,
    /// GitHub 之外的内部错误（Transport/Response）怎么回：
    /// Go 对文章是纯文本 500，对友链/碎语/相册是 400 `INVALID_*_PUBLISH`。
    internal_status: StatusCode,
    internal_code: &'a str,
}

async fn publish_error(
    state: &AppState,
    headers: &HeaderMap,
    failure: PublishFailure<'_>,
    error: crate::github_publish::PublishError,
) -> Response {
    let detail = match &error {
        crate::github_publish::PublishError::NotConfigured => "not_configured".to_string(),
        crate::github_publish::PublishError::Invalid(_) => "invalid_request".to_string(),
        crate::github_publish::PublishError::GitHub { status, .. } => {
            format!("github_status={status}")
        }
        crate::github_publish::PublishError::Transport(_)
        | crate::github_publish::PublishError::Response(_) => "internal_error".to_string(),
    };
    if let Err(response) = note_audit(
        state,
        headers,
        AuditNote {
            event: failure.event,
            outcome: "failure",
            actor: failure.actor,
            target_type: "owner_publish",
            target_id: failure.target,
            detail: &detail,
        },
    )
    .await
    {
        return *response;
    }
    match error {
        crate::github_publish::PublishError::NotConfigured => fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "PUBLISH_NOT_CONFIGURED",
            "站长发布尚未在服务器配置。",
        ),
        crate::github_publish::PublishError::Invalid(message) => {
            fail(StatusCode::BAD_REQUEST, failure.invalid_code, message)
        }
        // Go 把 GitHub 409/422 映射成 409（owner_publish.go:149-152），其余 GitHub 状态 502。
        crate::github_publish::PublishError::GitHub { status, message } => {
            let mapped = if status == 409 || status == 422 {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_GATEWAY
            };
            fail(mapped, failure.github_code, &message)
        }
        crate::github_publish::PublishError::Transport(message)
        | crate::github_publish::PublishError::Response(message) => {
            fail(failure.internal_status, failure.internal_code, &message)
        }
    }
}

#[derive(Deserialize)]
struct PostDeleteQuery {
    #[serde(default)]
    path: String,
}

#[derive(Deserialize)]
struct JournalDeleteQuery {
    #[serde(default)]
    slug: String,
}

#[derive(Deserialize)]
struct JournalBody {
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: String,
    #[serde(rename = "coverUrl", default)]
    cover_url: String,
}

#[derive(Deserialize)]
struct GalleryDeleteBody {
    #[serde(rename = "albumId", default)]
    album_id: String,
    #[serde(rename = "imageUrl", default)]
    image_url: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MomentDeleteBody {
    #[serde(default)]
    year: String,
    #[serde(default)]
    date: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct AboutBody {
    #[serde(default)]
    title: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    image: String,
}

#[derive(Deserialize)]
struct IdBody {
    #[serde(default)]
    id: String,
}

#[derive(Deserialize)]
struct BiliBody {
    #[serde(default)]
    bvid: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    cover: String,
    #[serde(default)]
    duration: String,
    #[serde(rename = "publishedAt", default)]
    published_at: String,
}

#[derive(Deserialize)]
struct BiliDeleteBody {
    #[serde(default)]
    bvid: String,
}

#[derive(Deserialize)]
struct ProjectBody {
    #[serde(default)]
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    cover: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    tags: String,
}

async fn owner_post_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PostDeleteQuery>,
) -> Response {
    let raw = query
        .path
        .trim()
        .strip_prefix("site_entries/")
        .unwrap_or(query.path.trim());
    let Ok(id) = raw.parse::<i64>() else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_POST", "文章编号无效");
    };
    remove_entry(
        &state,
        &headers,
        "owner.post_delete",
        "INVALID_POST",
        id,
        "article",
    )
    .await
}

#[derive(Deserialize)]
struct KindQuery {
    #[serde(default)]
    kind: String,
}

async fn owner_content_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(query): Query<KindQuery>,
) -> Response {
    remove_entry(
        &state,
        &headers,
        "owner.content_delete",
        "CONTENT_NOT_FOUND",
        id,
        &query.kind,
    )
    .await
}

async fn owner_journal(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<JournalBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    save_entry(
        &state,
        &headers,
        &owner,
        "owner.journal_publish",
        "INVALID_JOURNAL",
        crate::site_store::NewEntry {
            kind: "journal".into(),
            title: body.title,
            body: body.body,
            image_url: body.cover_url,
            extra: json!({}),
        },
        0,
    )
    .await
}
async fn owner_journal_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<JournalDeleteQuery>,
) -> Response {
    let Ok(id) = query.slug.parse::<i64>() else {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_JOURNAL",
            "建站日志编号无效",
        );
    };
    remove_entry(
        &state,
        &headers,
        "owner.journal_delete",
        "INVALID_JOURNAL",
        id,
        "journal",
    )
    .await
}
async fn owner_gallery_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<GalleryDeleteBody>,
) -> Response {
    let Ok(id) = body
        .image_url
        .parse::<i64>()
        .or_else(|_| body.album_id.parse::<i64>())
    else {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_GALLERY_PUBLISH",
            "相册图片编号无效",
        );
    };
    remove_entry(
        &state,
        &headers,
        "owner.gallery_delete",
        "INVALID_GALLERY_PUBLISH",
        id,
        "gallery",
    )
    .await
}
async fn owner_moment_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<MomentDeleteBody>,
) -> Response {
    let Ok(id) = body.content.parse::<i64>() else {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_MOMENT_PUBLISH",
            "碎语编号无效",
        );
    };
    remove_entry(
        &state,
        &headers,
        "owner.moment_delete",
        "INVALID_MOMENT_PUBLISH",
        id,
        "moment",
    )
    .await
}
async fn owner_about(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<AboutBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    save_entry(
        &state,
        &headers,
        &owner,
        "owner.about_publish",
        "INVALID_ABOUT",
        crate::site_store::NewEntry {
            kind: "about".into(),
            title: body.title,
            body: body.text,
            image_url: body.image,
            extra: json!({}),
        },
        0,
    )
    .await
}
async fn owner_about_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<IdBody>,
) -> Response {
    let Ok(id) = body.id.parse::<i64>() else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_ABOUT", "关于我编号无效");
    };
    remove_entry(
        &state,
        &headers,
        "owner.about_delete",
        "INVALID_ABOUT",
        id,
        "about",
    )
    .await
}
async fn owner_bili(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<BiliBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let published = if body.published_at.trim().is_empty() {
        (Utc::now() + chrono::Duration::hours(8))
            .format("%Y-%m-%d")
            .to_string()
    } else {
        body.published_at
    };
    save_entry(
        &state,
        &headers,
        &owner,
        "owner.bili_publish",
        "INVALID_BILI",
        crate::site_store::NewEntry {
            kind: "bili".into(),
            title: body.title,
            body: body.description,
            image_url: body.cover,
            extra: json!({
                "bvid": body.bvid.trim(),
                "duration": body.duration.trim(),
                "publishedAt": published,
            }),
        },
        0,
    )
    .await
}
async fn owner_bili_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<BiliDeleteBody>,
) -> Response {
    let Ok(id) = body.bvid.parse::<i64>() else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_BILI", "哔哩视频编号无效");
    };
    remove_entry(
        &state,
        &headers,
        "owner.bili_delete",
        "INVALID_BILI",
        id,
        "bili",
    )
    .await
}
async fn owner_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ProjectBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    save_entry(
        &state,
        &headers,
        &owner,
        "owner.project_publish",
        "INVALID_PROJECT",
        crate::site_store::NewEntry {
            kind: "project".into(),
            title: body.title,
            body: body.summary,
            image_url: body.cover,
            extra: json!({"url": body.url.trim(), "tags": body.tags.trim()}),
        },
        0,
    )
    .await
}
async fn owner_project_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<IdBody>,
) -> Response {
    let Ok(id) = body.id.parse::<i64>() else {
        return fail(StatusCode::BAD_REQUEST, "INVALID_PROJECT", "星空馆编号无效");
    };
    remove_entry(
        &state,
        &headers,
        "owner.project_delete",
        "INVALID_PROJECT",
        id,
        "project",
    )
    .await
}
async fn owner_gallery(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<GalleryBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let title = if body.album_title.trim().is_empty() {
        body.album_id.clone()
    } else {
        body.album_title.clone()
    };
    let (album_id, album_title, image_url) = (
        body.album_id.clone(),
        body.album_title.clone(),
        body.image_url.clone(),
    );
    let response = save_entry(
        &state,
        &headers,
        &owner,
        "owner.gallery_publish",
        "INVALID_GALLERY_PUBLISH",
        crate::site_store::NewEntry {
            kind: "gallery".into(),
            title,
            body: String::new(),
            image_url: body.image_url,
            extra: json!({"albumId": album_id, "albumTitle": album_title}),
        },
        0,
    )
    .await;
    if response.status().is_success() {
        dispatch_publish_extras(
            &state,
            json!({"kind": "gallery", "albumId": album_id, "albumTitle": album_title, "imageUrl": image_url}),
        )
        .await;
    }
    response
}
async fn owner_friends(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<FriendBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    if !public_http_url(body.url.trim()) {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_FRIEND_PUBLISH",
            "友链 URL 必须是公开的 http 或 https 地址。",
        );
    }
    let response = save_entry(
        &state,
        &headers,
        &owner,
        "owner.friend_publish",
        "INVALID_FRIEND_PUBLISH",
        crate::site_store::NewEntry {
            kind: "friend".into(),
            title: body.name.clone(),
            body: body.desc.clone(),
            image_url: body.avatar.clone(),
            extra: json!({"url": body.url, "desc": body.desc}),
        },
        0,
    )
    .await;
    if response.status().is_success() {
        dispatch_publish_extras(
            &state,
            json!({"kind": "friend", "name": body.name, "desc": body.desc, "url": body.url, "avatar": body.avatar}),
        )
        .await;
    }
    response
}

async fn owner_moments(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<MomentBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let kind = if body.kind.trim().is_empty() {
        body.category.clone()
    } else {
        body.kind.clone()
    };
    let now = Utc::now() + chrono::Duration::hours(8);
    let year = if body.year.trim().is_empty() {
        now.format("%Y").to_string()
    } else {
        body.year.clone()
    };
    let date = if body.date.trim().is_empty() {
        format!(
            "{}.{}",
            now.format("%m").to_string().trim_start_matches('0'),
            now.format("%d").to_string().trim_start_matches('0')
        )
    } else {
        body.date.clone()
    };
    let (m_year, m_date, m_kind, m_category, m_content) = (
        year.clone(),
        date.clone(),
        kind.clone(),
        body.category.clone(),
        body.content.clone(),
    );
    let response = save_entry(
        &state,
        &headers,
        &owner,
        "owner.moment_publish",
        "INVALID_MOMENT_PUBLISH",
        crate::site_store::NewEntry {
            kind: "moment".into(),
            title: kind.clone(),
            body: body.content,
            image_url: String::new(),
            extra: json!({"year": year, "date": date, "type": kind}),
        },
        0,
    )
    .await;
    if response.status().is_success() {
        dispatch_publish_extras(
            &state,
            json!({"kind": "moment", "year": m_year, "date": m_date, "type": m_kind, "category": m_category, "content": m_content}),
        )
        .await;
    }
    response
}
async fn fixed_answers(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if require_owner(&state, &headers).await.is_err() {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
        "SELECT id, question, answer, status, created_at, updated_at FROM ai_fixed_answers WHERE status = 'active' ORDER BY updated_at DESC, id DESC LIMIT 50",
    )
    .fetch_all(pool)
    .await;
    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let items: Vec<Value> = rows.into_iter().map(|(id, question, answer, status, created, updated)| json!({"id": id, "question": question, "answer": answer, "status": status, "createdAt": go_timestamp(created), "updatedAt": go_timestamp(updated)})).collect();
    Json(json!({"items": items})).into_response()
}

async fn fixed_answer_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<Value>,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let question = body.get("question").and_then(Value::as_str).unwrap_or("");
    let answer = body.get("answer").and_then(Value::as_str).unwrap_or("");
    if !crate::fixed_answers::input_ok(question, answer) {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_FIXED_ANSWER",
            "固定答案需要填写问题和答案，且内容不能过长。",
        );
    }
    let question = question.trim();
    let answer = answer.trim();
    let normalized = crate::fixed_answers::normalize_question(question);
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>, DateTime<Utc>)>(
        "INSERT INTO ai_fixed_answers (question, normalized_question, answer, status) VALUES ($1, $2, $3, 'active') ON CONFLICT (normalized_question) WHERE normalized_question <> '' DO UPDATE SET question = EXCLUDED.question, answer = EXCLUDED.answer, status = 'active', updated_at = now() RETURNING id, created_at, updated_at",
    )
    .bind(question)
    .bind(normalized)
    .bind(answer)
    .fetch_one(pool)
    .await;
    let Ok((id, created, updated)) = row else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "固定答案没有保存",
        );
    };
    Json(json!({"ok": true, "item": {"id": id, "question": question, "answer": answer, "status": "active", "createdAt": go_timestamp(created), "updatedAt": go_timestamp(updated)}})).into_response()
}

async fn notification_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let updated = sqlx::query(
        "UPDATE guestbook_messages SET owner_read_at = now() WHERE id = $1 AND status = 'visible'",
    )
    .bind(id)
    .execute(pool)
    .await;
    if updated.map(|result| result.rows_affected()).unwrap_or(0) == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "提醒不存在。");
    }
    Json(json!({"ok": true, "id": id, "ownerReadAt": go_timestamp(Utc::now())})).into_response()
}

#[derive(Deserialize)]
struct PresignBody {
    key: Option<String>,
    #[serde(rename = "contentType")]
    content_type: Option<String>,
}

async fn owner_presign(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<PresignBody>,
) -> Response {
    if require_owner(&state, &headers).await.is_err() {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    }
    if !state.cfg.r2_ready() {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "R2_NOT_CONFIGURED",
            "R2 还没有配置",
        );
    }
    let key = body
        .key
        .unwrap_or_else(|| format!("uploads/{}.bin", hex::encode(rand::random::<[u8; 8]>())));
    let url = presign_put(&state.cfg, &key, 600);
    Json(json!({"ok": true, "url": url, "key": key, "publicUrl": public_url(&state.cfg, &key), "contentType": body.content_type.unwrap_or_else(|| "application/octet-stream".into())})).into_response()
}

async fn owner_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    match store_upload(&state, multipart, "uploads").await {
        Ok((public, key, name, size, mime)) => Json(json!({"ok": true, "item": {"name": name, "url": public, "path": key, "size": size, "mimeType": mime}})).into_response(),
        Err(response) => *response,
    }
}

async fn owner_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let mut bytes = Bytes::new();
    let mut name = String::from("upload.bin");
    let mut kind = String::new();
    let mut album = String::new();
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return fail(
                    StatusCode::BAD_REQUEST,
                    "INVALID_UPLOAD",
                    &error.to_string(),
                )
            }
        };
        match field.name() {
            Some("file") => {
                if let Some(file_name) = field.file_name() {
                    name = file_name.to_string();
                }
                bytes = match field.bytes().await {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        return fail(
                            StatusCode::BAD_REQUEST,
                            "INVALID_UPLOAD",
                            &error.to_string(),
                        )
                    }
                };
            }
            Some("kind") => {
                kind = match field.text().await {
                    Ok(value) => value,
                    Err(error) => {
                        return fail(
                            StatusCode::BAD_REQUEST,
                            "INVALID_UPLOAD",
                            &error.to_string(),
                        )
                    }
                };
            }
            Some("album") => {
                album = match field.text().await {
                    Ok(value) => value,
                    Err(error) => {
                        return fail(
                            StatusCode::BAD_REQUEST,
                            "INVALID_UPLOAD",
                            &error.to_string(),
                        )
                    }
                };
            }
            _ => {}
        }
    }
    if kind == "gallery" && album.trim().is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_ALBUM",
            "相册上传必须选择相册。",
        );
    }
    let album_slug: String = album
        .trim()
        .chars()
        .map(|ch| if ch == '/' || ch == '\\' { '-' } else { ch })
        .collect();
    let folder = match kind.as_str() {
        "gallery" => format!("gallery/{album_slug}"),
        "article" => {
            if album_slug.is_empty() {
                "articles".to_string()
            } else {
                format!("articles/{album_slug}")
            }
        }
        "about" => "about".to_string(),
        "journal" => "journal".to_string(),
        "bili" => "bili".to_string(),
        "project" => "projects".to_string(),
        _ => {
            return fail(
                StatusCode::BAD_REQUEST,
                "INVALID_ASSET_KIND",
                "资源类型必须是关于我、相册、哔哩、文章、建站日志或星空馆。",
            )
        }
    };
    match put_bytes(&state, &folder, &name, bytes).await {
        Ok((public, key, name, size, mime)) => Json(json!({"ok": true, "item": {"kind": kind, "url": public, "path": key, "name": name, "size": size, "mimeType": mime}})).into_response(),
        Err(response) => *response,
    }
}

async fn store_upload(
    state: &AppState,
    mut multipart: Multipart,
    folder: &str,
) -> Result<(String, String, String, usize, &'static str), Box<Response>> {
    let mut bytes = Bytes::new();
    let mut name = String::from("upload.bin");
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return Err(Box::new(fail(
                    StatusCode::BAD_REQUEST,
                    "INVALID_UPLOAD",
                    &error.to_string(),
                )))
            }
        };
        if field.name() == Some("file") {
            if let Some(file_name) = field.file_name() {
                name = file_name.to_string();
            }
            bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(error) => {
                    return Err(Box::new(fail(
                        StatusCode::BAD_REQUEST,
                        "INVALID_UPLOAD",
                        &error.to_string(),
                    )))
                }
            };
        }
    }
    put_bytes(state, folder, &name, bytes).await
}

async fn put_bytes(
    state: &AppState,
    folder: &str,
    _name: &str,
    bytes: Bytes,
) -> Result<(String, String, String, usize, &'static str), Box<Response>> {
    if !state.cfg.r2_ready() {
        return Err(Box::new(fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "R2_NOT_CONFIGURED",
            "R2 还没有配置",
        )));
    }
    if bytes.is_empty() {
        return Err(Box::new(fail(
            StatusCode::BAD_REQUEST,
            "MISSING_FILE",
            "请选择要上传的文件。",
        )));
    }
    // 头像和其它图片共用 8MB 上限。头像仍用自己的类型错误码。
    let (max_bytes, too_large_code, too_large_msg, invalid_code, invalid_msg) =
        if folder == "avatars" {
            (
                8 * 1024 * 1024,
                "FILE_TOO_LARGE",
                "上传文件超过 8 MiB 限制。",
                "INVALID_IMAGE_TYPE",
                "仅支持 PNG / JPEG / WebP / GIF",
            )
        } else {
            (
                8 * 1024 * 1024,
                "FILE_TOO_LARGE",
                "上传文件超过 8 MiB 限制。",
                "INVALID_FILE_TYPE",
                "当前只支持上传图片。",
            )
        };
    if bytes.len() > max_bytes {
        return Err(Box::new(fail(
            StatusCode::BAD_REQUEST,
            too_large_code,
            too_large_msg,
        )));
    }
    let Some(mime) = image_mime(&bytes) else {
        return Err(Box::new(fail(
            StatusCode::BAD_REQUEST,
            invalid_code,
            invalid_msg,
        )));
    };
    // 文件名重命名为时间戳+随机串，避免同名覆盖；扩展名按嗅探结果定。
    let ext = match mime {
        "image/png" => ".png",
        "image/jpeg" => ".jpg",
        "image/webp" => ".webp",
        _ => ".gif",
    };
    let safe_name = format!(
        "{}-{}{ext}",
        Utc::now().format("%Y%m%d%H%M%S"),
        hex::encode(rand::random::<[u8; 4]>())
    );
    let key = format!("{folder}/{}", safe_name);
    let size = bytes.len();
    // R2 预签名只认 UNSIGNED-PAYLOAD，正文哈希进签名会 403（和 image_store 同款坑）。
    let url = presign_put(&state.cfg, &key, 300);
    let sent = state
        .http
        .put(url)
        .header(header::CONTENT_TYPE, HeaderValue::from_static(mime))
        .body(bytes)
        .send()
        .await;
    if sent.and_then(|item| item.error_for_status()).is_err() {
        return Err(Box::new(fail(
            StatusCode::BAD_GATEWAY,
            "R2_ERROR",
            "上传到 R2 失败",
        )));
    }
    Ok((public_url(&state.cfg, &key), key, safe_name, size, mime))
}

fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

async fn owner_draft_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)>("SELECT id, kind, title, body, cover_url, status, created_at, updated_at FROM owner_drafts ORDER BY updated_at DESC, id DESC LIMIT 20")
        .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, kind, title, body, cover, status, created, updated)| json!({"id": id, "kind": kind, "title": title, "body": body, "coverUrl": cover, "status": status, "createdAt": go_timestamp(created), "updatedAt": go_timestamp(updated)})).collect())
}

async fn fixed_answer_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
        "SELECT id, question, answer, status, created_at, updated_at FROM ai_fixed_answers WHERE status = 'active' ORDER BY updated_at DESC, id DESC LIMIT 50",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id, question, answer, status, created, updated)| json!({"id": id, "question": question, "answer": answer, "status": status, "createdAt": go_timestamp(created), "updatedAt": go_timestamp(updated)})).collect())
}

async fn notification_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>, String)>(
        "SELECT gm.id, gm.channel, gm.nickname, gm.content, gm.contact_email, COALESCE(u.email, ''), gm.created_at, gm.extra::text FROM guestbook_messages gm LEFT JOIN users u ON u.id = gm.user_id WHERE gm.status = 'visible' AND gm.owner_read_at IS NULL ORDER BY gm.id DESC LIMIT 30",
    )
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                channel,
                nickname,
                content,
                contact_email,
                account_email,
                created,
                extra_text,
            )| {
                let submitted = contact_email.trim().to_lowercase();
                let account = account_email.trim().to_lowercase();
                let contact = if submitted.is_empty() {
                    account
                } else {
                    submitted
                };
                let extra: Value = serde_json::from_str(&extra_text).unwrap_or_else(|_| json!({}));
                let mut item = json!({
                    "id": id,
                    "source": channel,
                    "title": crate::guestbook::notice_title(&channel),
                    "detail": crate::guestbook::notice_detail(&channel, &nickname, created),
                    "nickname": nickname,
                    "content": content,
                    "contactEmail": contact,
                    "createdAt": crate::guestbook::china_clock(created),
                    "count": 1
                });
                if extra.as_object().is_some_and(|map| !map.is_empty()) {
                    item["extra"] = extra;
                }
                item
            },
        )
        .collect())
}

async fn maintain(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    match crate::jobs::maintain(&state).await {
        Ok(report) => Json(json!({"ok": true, "report": report})).into_response(),
        Err(error) => fail(
            StatusCode::BAD_GATEWAY,
            "MAINTENANCE_FAILED",
            &error.to_string(),
        ),
    }
}

/// QStash 每天调一次：重新检测 OpenRouter 免费模型并随机换一个。
async fn refresh_ai_model(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    match crate::ai_model::refresh(&state).await {
        Ok(model) => Json(json!({"ok": true, "model": model})).into_response(),
        Err(error) => fail(StatusCode::BAD_GATEWAY, "MODEL_REFRESH_FAILED", &error),
    }
}

async fn index_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    match crate::jobs::index_content(&state).await {
        Ok(count) => Json(json!({"ok": true, "count": count})).into_response(),
        Err(error) => fail(StatusCode::BAD_GATEWAY, "INDEX_FAILED", &error.to_string()),
    }
}

async fn restock_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: OriginalUri,
    Path(id): Path<i64>,
    body: Bytes,
) -> Response {
    if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
        return *response;
    }
    match crate::jobs::restock_image(&state, id).await {
        Ok(item) => Json(json!({"ok": true, "item": item})).into_response(),
        Err(crate::jobs::JobError::NotFound) => {
            fail(StatusCode::NOT_FOUND, "NOT_FOUND", "图片不存在")
        }
        Err(crate::jobs::JobError::NoDatabase) => fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        ),
        Err(error) => fail(
            StatusCode::BAD_GATEWAY,
            "IMAGE_JOB_FAILED",
            &error.to_string(),
        ),
    }
}

fn scheduler_authorized(
    state: &AppState,
    headers: &HeaderMap,
    uri: &OriginalUri,
    body: &[u8],
) -> Result<(), Box<Response>> {
    if state.cfg.qstash_signing_key.is_empty() {
        return if state.cfg.allow_unsigned_internal {
            Ok(())
        } else {
            Err(Box::new(fail(
                StatusCode::UNAUTHORIZED,
                "BAD_SIGNATURE",
                "调度签名没有配置",
            )))
        };
    }
    let origin = state.cfg.public_origin.trim_end_matches('/');
    if origin.is_empty() {
        return Err(Box::new(fail(
            StatusCode::UNAUTHORIZED,
            "BAD_SIGNATURE",
            "缺少公开站点地址，无法核对调度目标",
        )));
    }
    let path = uri
        .path_and_query()
        .map(axum::http::uri::PathAndQuery::as_str)
        .unwrap_or(uri.path());
    let url = format!("{origin}{path}");
    let signature = match headers.get("upstash-signature") {
        None => {
            return Err(Box::new(fail(
                StatusCode::UNAUTHORIZED,
                "BAD_SIGNATURE",
                "缺少调度签名",
            )))
        }
        Some(value) => match value.to_str() {
            Ok(text) => text,
            Err(error) => {
                return Err(Box::new(fail(
                    StatusCode::UNAUTHORIZED,
                    "BAD_SIGNATURE",
                    &error.to_string(),
                )))
            }
        },
    };
    let input = crate::qstash::VerifyInput {
        current_key: &state.cfg.qstash_signing_key,
        next_key: &state.cfg.qstash_next_signing_key,
        signature,
        body,
        url: &url,
        now_unix: Utc::now().timestamp(),
    };
    crate::qstash::verify(&input).map_err(|error| {
        Box::new(fail(
            StatusCode::UNAUTHORIZED,
            "BAD_SIGNATURE",
            &error.to_string(),
        ))
    })
}

async fn admit(
    state: &AppState,
    headers: &HeaderMap,
    gate: &crate::rate_limit::RateGate,
    bucket: &str,
    limit: i64,
) -> Result<(), Box<Response>> {
    let ip = client_ip(headers);
    if state.cfg.redis_ready() {
        // 滑动窗口按分数记时间戳，key 不再带分钟桶。
        let key = format!("rl:{bucket}:{ip}");
        return match crate::rate_limit::allow_sliding_window(
            &state.http,
            &state.cfg.redis_url,
            &state.cfg.redis_token,
            &key,
            limit,
        )
        .await
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(Box::new(rate_limited())),
            Err(error) => {
                // Redis 故障时不能把用户挡在门外（Go 的内存限流从不失败）：
                // 降级放行，只记警告。
                tracing::warn!(%error, bucket, "rate limit backend failed; allowing request");
                Ok(())
            }
        };
    }
    if gate.allow(&ip) {
        Ok(())
    } else {
        Err(Box::new(rate_limited()))
    }
}

fn auth_limited(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    let ip = client_ip(headers);
    if state.auth_gate.allow(&ip) {
        None
    } else {
        Some(fail(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "操作太频繁，请稍后再试",
        ))
    }
}

fn rate_limited() -> Response {
    fail(
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMITED",
        "Too many requests, please try again later.",
    )
}

struct AuditNote<'a> {
    event: &'a str,
    outcome: &'a str,
    actor: i64,
    target_type: &'a str,
    target_id: &'a str,
    detail: &'a str,
}

async fn note_audit(
    state: &AppState,
    headers: &HeaderMap,
    note: AuditNote<'_>,
) -> Result<(), Box<Response>> {
    // Go 的安全审计从不影响业务响应；这里失败也只记日志。
    let Some(pool) = state.db.as_ref() else {
        tracing::warn!(event = note.event, "audit skipped: database not connected");
        return Ok(());
    };
    let user_agent = match headers.get(header::USER_AGENT) {
        None => String::new(),
        Some(value) => value
            .to_str()
            .map(str::to_string)
            .unwrap_or_else(|_| "invalid-user-agent".to_string()),
    };
    let ip = client_ip(headers);
    if let Err(error) = crate::audit::record(
        pool,
        crate::audit::AuditEvent {
            event: note.event,
            outcome: note.outcome,
            actor_user_id: note.actor,
            ip: &ip,
            user_agent: &user_agent,
            target_type: note.target_type,
            target_id: note.target_id,
            detail: note.detail,
        },
    )
    .await
    {
        tracing::warn!(%error, event = note.event, "audit write failed");
    }
    Ok(())
}

#[allow(dead_code)]
fn _rate_window(last: &mut Instant) -> bool {
    let now = Instant::now();
    if now.duration_since(*last) < Duration::from_secs(1) {
        return false;
    }
    *last = now;
    true
}

/// 公开 http(s) 链接校验：友链地址与头像必须是可公开访问的网址。
fn public_http_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.rsplit('@').next().unwrap_or("");
    !host.is_empty() && !host.starts_with(':')
}

async fn client_deploy_status(State(state): State<AppState>) -> Response {
    let mut body = crate::deploy_status::snapshot(&state).await;
    // 只暴露「是否配置」，不暴露凭据本身；前端据此区分未配置与读取失败。
    body["githubConfigured"] =
        json!(!state.cfg.github_token.is_empty() && !state.cfg.github_repo.is_empty());
    body["aiConfigured"] = json!(!state.cfg.deepseek_api_key.is_empty());
    Json(body).into_response()
}

#[derive(Deserialize)]
struct FriendApproveBody {
    id: i64,
}

/// 收件箱「通过」：把 friends 分区的申请转成友链条目，朋友页实时展示。
/// 结构化 extra 优先，其次按四行规则解析正文；GitHub 配置存在时直写主分支同步源码。
async fn owner_friend_approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<FriendApproveBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let pool = match db_required(&state) {
        Ok(pool) => pool,
        Err(response) => return *response,
    };
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT nickname, content, extra::text, channel FROM guestbook_messages WHERE id = $1 AND status = 'visible'",
    )
    .bind(body.id)
    .fetch_optional(pool)
    .await;
    let Ok(Some((nickname, content, extra_text, channel))) = row else {
        return fail(
            StatusCode::NOT_FOUND,
            "APPLICATION_NOT_FOUND",
            "这条申请不存在或已隐藏。",
        );
    };
    if channel != "friends" {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_CHANNEL",
            "只有朋友页申请可以一键通过。",
        );
    }
    let extra: Value = serde_json::from_str(&extra_text).unwrap_or_else(|_| json!({}));
    let pick = |key: &str| {
        extra
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("")
            .to_string()
    };
    let parsed = parse_friend_lines(&content);
    let mut url = pick("url");
    if url.is_empty() {
        url = parsed.2.clone();
    }
    if !public_http_url(&url) {
        return fail(
            StatusCode::BAD_REQUEST,
            "NO_URL",
            "申请里没有有效的站点 URL，请改用友链表单手动填写。",
        );
    }
    let mut name = pick("name");
    if name.is_empty() {
        name = parsed.0.clone();
    }
    if name.is_empty() {
        name = nickname.trim().to_string();
    }
    if name.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "NO_NAME",
            "申请里没有站名，请改用友链表单手动填写。",
        );
    }
    let mut desc = pick("desc");
    if desc.is_empty() {
        desc = parsed.1.clone();
    }
    let mut avatar = pick("avatar");
    if avatar.is_empty() || !public_http_url(&avatar) {
        avatar = parsed.3.clone();
    }
    if !avatar.is_empty() && !public_http_url(&avatar) {
        avatar = String::new();
    }
    // 同 URL 去重：静态卡片或已通过条目里已有就不再落库。
    let existing = crate::site_store::list(pool, "friend")
        .await
        .unwrap_or_default();
    let normalized = url.trim_end_matches('/').to_lowercase();
    let duplicate = existing.iter().any(|entry| {
        entry
            .extra
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase()
            == normalized
    });
    let entry_id = if duplicate {
        0
    } else {
        match crate::site_store::insert(
            pool,
            crate::site_store::NewEntry {
                kind: "friend".to_string(),
                title: name.clone(),
                body: desc.clone(),
                image_url: avatar.clone(),
                extra: json!({ "url": url, "desc": desc, "applicationId": body.id }),
            },
        )
        .await
        {
            Ok(saved) => saved.id,
            Err(error) => return content_error(error, "FRIEND_APPROVE_FAILED"),
        }
    };
    let _ = sqlx::query("UPDATE guestbook_messages SET owner_read_at = now() WHERE id = $1")
        .bind(body.id)
        .execute(pool)
        .await;
    // GitHub 同步是可选增强：未配置或失败都不阻塞「通过」，页面靠数据库实时展示。
    let mut github_synced = false;
    let mut github_note = String::from("未配置 GitHub，朋友页已实时展示");
    let branch = if state.cfg.github_branch.is_empty() {
        "main"
    } else {
        state.cfg.github_branch.as_str()
    };
    if let Ok((gh_owner, gh_repo, gh_branch)) = crate::github_publish::repo_from_config(
        &state.cfg.github_token,
        &state.cfg.github_repo,
        branch,
    ) {
        let spec = crate::github_publish::RepoSpec {
            api_base: "https://api.github.com",
            token: &state.cfg.github_token,
            owner: gh_owner,
            repo: gh_repo,
            branch: gh_branch,
        };
        let input = crate::js_source::FriendInput {
            name: &name,
            desc: &desc,
            url: &url,
            avatar: &avatar,
        };
        match crate::github_publish::publish_friend_direct(&state.http, &spec, input, Utc::now())
            .await
        {
            Ok(_) => {
                github_synced = true;
                github_note = format!("已直写 {gh_branch} 分支，无需合并 PR");
            }
            Err(error) => {
                github_note = format!("GitHub 同步失败（{error}），朋友页已实时展示");
            }
        }
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "owner.friend_approve",
            outcome: "success",
            actor: owner.id,
            target_type: "guestbook_message",
            target_id: &body.id.to_string(),
            detail: "approved",
        },
    )
    .await
    {
        return *response;
    }
    Json(json!({
        "ok": true,
        "duplicate": duplicate,
        "entryId": entry_id,
        "githubSynced": github_synced,
        "githubNote": github_note,
        "item": { "name": name, "url": url, "desc": desc, "avatar": avatar },
    }))
    .into_response()
}

/// 四行友链文本解析（与前端 friendQuickInput 同款规则）：
/// 返回 (站名, 描述, 站点URL, 头像URL)。带标签行优先，其余按评分挑选。
fn parse_friend_lines(content: &str) -> (String, String, String, String) {
    let mut name = String::new();
    let mut desc = String::new();
    let mut url = String::new();
    let mut avatar = String::new();
    let mut text_lines: Vec<String> = Vec::new();
    let mut url_lines: Vec<String> = Vec::new();
    for raw in content.lines() {
        let line: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            continue;
        }
        if let Some((label, value)) = split_friend_label(&line) {
            let matched = match label.as_str() {
                "name" | "名称" | "站名" | "站点" | "网站" | "博客" => {
                    name = value.clone();
                    true
                }
                "desc" | "description" | "描述" | "简介" | "介绍" | "说明" => {
                    desc = value.clone();
                    true
                }
                "url" | "link" | "链接" | "地址" | "网址" | "站点地址" | "网站地址" =>
                {
                    url = value.clone();
                    true
                }
                "avatar" | "头像" | "图标" | "logo" | "图片" => {
                    avatar = value.clone();
                    true
                }
                _ => false,
            };
            if matched {
                continue;
            }
        }
        if public_http_url(&line) {
            url_lines.push(line);
        } else {
            text_lines.push(line);
        }
    }
    for value in url_lines {
        if looks_like_avatar(&value) && avatar.is_empty() {
            avatar = value;
        } else if url.is_empty() {
            url = value;
        } else if avatar.is_empty() {
            avatar = value;
        }
    }
    let mut remaining: Vec<String> = text_lines
        .into_iter()
        .filter(|line| line != &name && line != &desc)
        .collect();
    if name.is_empty() && !remaining.is_empty() {
        remaining.sort_by_key(|line| {
            std::cmp::Reverse(friend_name_score(line) - friend_desc_score(line))
        });
        name = remaining.remove(0);
    }
    if desc.is_empty() && !remaining.is_empty() {
        remaining.sort_by_key(|line| std::cmp::Reverse(friend_desc_score(line)));
        desc = remaining.remove(0);
    }
    (name, desc, url, avatar)
}

fn split_friend_label(line: &str) -> Option<(String, String)> {
    if public_http_url(line) {
        return None;
    }
    let (label, value) = line.split_once([':', '：'])?;
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 12 {
        return None;
    }
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some((label.to_lowercase(), value.to_string()))
}

fn looks_like_avatar(value: &str) -> bool {
    let lower = value.to_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or("");
    [
        ".avif", ".gif", ".ico", ".jpg", ".jpeg", ".png", ".svg", ".webp",
    ]
    .iter()
    .any(|ext| path.ends_with(ext))
        || ["avatar", "head", "logo", "icon", "favicon", "upload"]
            .iter()
            .any(|word| path.contains(word))
}

fn friend_name_score(value: &str) -> i32 {
    let mut score = 0;
    if value.chars().any(|c| "。.!！?？,，；;".contains(c)) {
        score -= 3;
    }
    let lower = value.to_lowercase();
    if [
        "blog", "博客", "小窝", "小屋", "主页", "站", "site", "log", "space", "空间", "の",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        score += 3;
    }
    if [
        "愿", "记录", "分享", "生活", "技术", "热爱", "欢迎", "一个", "这里",
    ]
    .iter()
    .any(|word| value.contains(word))
    {
        score -= 1;
    }
    if value.chars().count() <= 24 {
        score += 1;
    }
    score
}

fn friend_desc_score(value: &str) -> i32 {
    let mut score = 0;
    if value.chars().any(|c| "。.!！?？,，；;".contains(c)) {
        score += 2;
    }
    if [
        "愿", "记录", "分享", "生活", "技术", "热爱", "欢迎", "一个", "这里",
    ]
    .iter()
    .any(|word| value.contains(word))
    {
        score += 2;
    }
    score += (value.chars().count() as i32 / 20).min(2);
    score
}

#[derive(Deserialize)]
struct OwnerAiGenerateBody {
    kind: String,
    #[serde(default)]
    input: String,
}

/// 站长控制台的真实 AI 生成：文章草稿 / 固定答案候选。
/// 复用聊天的模型轮换链路，但不占用户聊天配额、不走用户侧系统提示词。
async fn owner_ai_generate(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<OwnerAiGenerateBody>,
) -> Response {
    if let Err(response) = require_owner(&state, &headers).await {
        return *response;
    }
    let kind = body.kind.trim();
    if kind != "draft" && kind != "answers" {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_KIND",
            "kind 只能是 draft 或 answers。",
        );
    }
    let input = body.input.trim();
    if input.is_empty() || input.chars().count() > 4000 {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_INPUT",
            "请填写素材或问题（4000 字以内）。",
        );
    }
    if state.cfg.deepseek_api_key.is_empty() {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "AI_NOT_CONFIGURED",
            "服务器尚未配置 AI 密钥。",
        );
    }
    let (system, user, max_tokens) = if kind == "draft" {
        (
            "你是博客站长的写作助手。根据站长提供的素材写一篇 Markdown 文章草稿：\
             第一行是「# 标题」，正文包含「## 素材摘要」与「## 发布建议」两个小节，\
             语气自然、不编造素材里没有的事实，全文 600 字以内。只输出 Markdown 正文。",
            format!("素材：\n{input}"),
            1200_u32,
        )
    } else {
        (
            "你是网站 AI 助手的调试助手。针对用户问题给出 3 条互不相同的候选回答，\
             每条 120 字以内、口语化、不编造事实。只输出一个 JSON 字符串数组，\
             例如 [\"回答一\",\"回答二\",\"回答三\"]，不要输出其它内容。",
            format!("用户问题：{input}"),
            800_u32,
        )
    };
    let endpoint = format!(
        "{}/v1/chat/completions",
        state.cfg.deepseek_base_url.trim_end_matches('/')
    );
    let mut model = crate::ai_model::resolve(&state).await;
    let mut attempts = 0_u8;
    let response = loop {
        attempts += 1;
        let sent = state
            .http
            .post(&endpoint)
            .bearer_auth(&state.cfg.deepseek_api_key)
            .timeout(Duration::from_secs(45))
            .json(&json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user},
                ],
                "temperature": 0.7,
                "max_tokens": max_tokens,
            }))
            .send()
            .await;
        let sent = match sent {
            Ok(item) => item,
            Err(error) => {
                return fail(
                    StatusCode::BAD_GATEWAY,
                    "AI_UPSTREAM",
                    &format!("模型请求失败: {error}"),
                );
            }
        };
        let status = sent.status();
        if status.is_success() {
            break sent;
        }
        let detail = sent.text().await.unwrap_or_default();
        let model_rejected = matches!(status.as_u16(), 403 | 404 | 429);
        if !model_rejected || attempts >= 3 {
            return fail(
                StatusCode::BAD_GATEWAY,
                "AI_UPSTREAM",
                &format!(
                    "HTTP {}: {}",
                    status,
                    detail.chars().take(200).collect::<String>()
                ),
            );
        }
        crate::ai_model::mark_failed(&state, &model);
        model = crate::ai_model::resolve(&state).await;
    };
    let payload: Value = match response.json().await {
        Ok(payload) => payload,
        Err(error) => {
            return fail(
                StatusCode::BAD_GATEWAY,
                "AI_UPSTREAM",
                &format!("模型响应无法解析: {error}"),
            );
        }
    };
    let Some(reply) = payload["choices"][0]["message"]["content"]
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    else {
        return fail(StatusCode::BAD_GATEWAY, "AI_UPSTREAM", "模型没有返回内容。");
    };
    if kind == "draft" {
        let title = reply
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("AI 草稿")
            .to_string();
        return Json(json!({ "ok": true, "title": title, "body": reply, "model": model }))
            .into_response();
    }
    let answers = parse_candidate_answers(reply);
    if answers.is_empty() {
        return fail(
            StatusCode::BAD_GATEWAY,
            "AI_UPSTREAM",
            "模型没有返回可用的候选答案。",
        );
    }
    Json(json!({ "ok": true, "answers": answers, "model": model })).into_response()
}

/// 候选答案优先按 JSON 数组解析；模型没守格式时按行拆兜底。
fn parse_candidate_answers(reply: &str) -> Vec<String> {
    let trimmed = reply.trim();
    let start = trimmed.find('[');
    let end = trimmed.rfind(']');
    if let (Some(start), Some(end)) = (start, end) {
        if end > start {
            if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&trimmed[start..=end]) {
                let answers: Vec<String> = items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_string)
                    .collect();
                if !answers.is_empty() {
                    return answers.into_iter().take(3).collect();
                }
            }
        }
    }
    reply
        .lines()
        .map(|line| {
            line.trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || matches!(c, '.' | '、' | ')' | '）' | '-' | '*' | ' ')
                })
                .trim()
                .to_string()
        })
        .filter(|text| !text.is_empty())
        .take(3)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_reports_missing_services() {
        let app = router(AppState::new(
            crate::config::Config::for_test(),
            None,
            reqwest::Client::new(),
        ));
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "degraded");
        assert_eq!(body["db"], "down");
        assert!(matches!(
            body["r2"].as_str(),
            Some("up" | "down" | "missing")
        ));
    }

    #[tokio::test]
    async fn auth_me_is_logged_out_without_a_database() {
        let app = router(AppState::new(
            crate::config::Config::for_test(),
            None,
            reqwest::Client::new(),
        ));
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["loggedIn"], false);
    }

    #[tokio::test]
    async fn site_avatar_is_empty_without_a_database() {
        let app = router(AppState::new(
            crate::config::Config::for_test(),
            None,
            reqwest::Client::new(),
        ));
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/site/avatar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["avatar"], "");
        assert_eq!(body["displayName"], "");
    }

    #[tokio::test]
    async fn chat_stats_and_bili_feed_keep_their_shape_without_a_database() {
        let app = router(AppState::new(
            crate::config::Config::for_test(),
            None,
            reqwest::Client::new(),
        ));
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/chat/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["configured"], false);
        assert_eq!(body["summary"]["successRateText"], "0%");
        assert_eq!(body["hourlyToday"].as_array().unwrap().len(), 24);
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/bili/videos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["ready"], false);
        assert_eq!(body["owner"]["uid"], "1061280173");
        assert_eq!(body["sync"]["state"], "pending");
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/acg/image/owner_BV1jK7W6MEAj.jpg")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn chat_summary_counts_today_and_the_period() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
        let daily = vec![
            (today, 2, 1, 0, 0, 0, 1, 2),
            (
                NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
                3,
                0,
                1,
                0,
                0,
                0,
                4,
            ),
        ];
        let body = chat_stats_json("deepseek-v4-flash", true, today, &daily, &[]);
        assert_eq!(body["summary"]["todaySuccess"], 2);
        assert_eq!(body["summary"]["todayTotal"], 3);
        assert_eq!(body["summary"]["periodSuccess"], 5);
        assert_eq!(body["summary"]["periodTotal"], 7);
        assert_eq!(body["summary"]["successRateText"], "71%");
    }

    #[test]
    fn quota_counts_fill_days_the_hourly_log_missed() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
        let mut daily = vec![(today, 1, 0, 0, 0, 0, 0, 1)];
        merge_quota_daily(
            &mut daily,
            &[
                (today, 4, 1, 3),
                (today - chrono::Duration::days(1), 2, 2, 0),
            ],
        );
        assert_eq!(daily[0].1, 2);
        assert_eq!(daily[1], (today, 4, 0, 0, 0, 0, 1, 3));
    }

    #[test]
    fn friend_lines_parse_labels_urls_and_scores() {
        let content = "站名: 小明的博客\n描述: 记录生活与技术\nhttps://example.com\nhttps://example.com/avatar.png";
        let (name, desc, url, avatar) = parse_friend_lines(content);
        assert_eq!(name, "小明的博客");
        assert_eq!(desc, "记录生活与技术");
        assert_eq!(url, "https://example.com");
        assert_eq!(avatar, "https://example.com/avatar.png");

        let free = "一个分享生活的小窝\n这里记录日常\nhttps://foo.bar";
        let (name2, _, url2, _) = parse_friend_lines(free);
        assert_eq!(url2, "https://foo.bar");
        assert!(!name2.is_empty());

        let (n, d, u, a) = parse_friend_lines("没有链接的普通留言");
        assert!(u.is_empty());
        assert!(a.is_empty());
        assert_eq!(n, "没有链接的普通留言");
        assert!(d.is_empty());
    }

    #[test]
    fn candidate_answers_parse_json_and_line_fallback() {
        let json_reply = "好的，候选如下：\n[\"答案一\", \"答案二\", \"答案三\", \"答案四\"]";
        assert_eq!(
            parse_candidate_answers(json_reply),
            vec!["答案一", "答案二", "答案三"]
        );
        let lines = "1. 第一条回答\n2、第二条回答\n- 第三条回答\n第四条多余";
        assert_eq!(
            parse_candidate_answers(lines),
            vec!["第一条回答", "第二条回答", "第三条回答"]
        );
        assert!(parse_candidate_answers("   ").is_empty());
    }

    #[test]
    fn upload_mime_recognizes_png_gif_and_webp() {
        assert_eq!(image_mime(&[0xFF, 0xD8, 0xFF, 0x00]), Some("image/jpeg"));
        assert_eq!(image_mime(b"\x89PNG\r\n\x1a\nrest"), Some("image/png"));
        assert_eq!(image_mime(b"GIF89a"), Some("image/gif"));
        let mut webp = b"RIFF....WEBP".to_vec();
        webp[4] = b'x';
        assert_eq!(image_mime(&webp), Some("image/webp"));
        assert_eq!(image_mime(b"not-an-image"), None);
    }
}
