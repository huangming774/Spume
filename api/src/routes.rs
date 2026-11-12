use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Multipart, OriginalUri, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::r2::{presign_put, public_url, signed_put_url};
use crate::AppState;

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state);
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/auth/me", get(auth_me))
        .route("/api/auth/register", post(auth_register))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/profile", axum::routing::patch(auth_profile))
        .route("/api/auth/verify-security", post(auth_verify_security))
        .route("/api/auth/avatar", post(auth_avatar))
        .route(
            "/api/guestbook/messages",
            get(guestbook_list).post(guestbook_create),
        )
        .route(
            "/api/guestbook/messages/{id}",
            axum::routing::delete(guestbook_delete).patch(guestbook_patch),
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
        .route("/api/owner/status", get(owner_status))
        .route("/api/owner/emails", get(owner_emails))
        .route(
            "/api/owner/drafts",
            get(owner_drafts).post(owner_draft_create),
        )
        .route("/api/owner/publish", post(owner_publish))
        .route("/api/owner/gallery/images", post(owner_gallery))
        .route("/api/owner/friends", post(owner_friends))
        .route("/api/owner/moments", post(owner_moments))
        .route(
            "/api/owner/ai/fixed-answers",
            get(fixed_answers).post(fixed_answer_create),
        )
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
        .route("/api/internal/jobs/image/{id}", post(restock_image))
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(cors)
        .with_state(state)
}

fn cors_layer(state: &AppState) -> CorsLayer {
    let origins: Vec<HeaderValue> = state
        .cfg
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    let origin = if origins.is_empty() {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(origins)
    };
    CorsLayer::new()
        .allow_origin(origin)
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
            HeaderName::from_static("x-sync-trigger-token"),
        ])
}

fn fail(status: StatusCode, error: &str, message: &str) -> Response {
    (status, Json(json!({"error": error, "message": message}))).into_response()
}

fn query_failed(error: &sqlx::Error) -> Response {
    fail(
        StatusCode::INTERNAL_SERVER_ERROR,
        "SERVER_ERROR",
        &error.to_string(),
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
}

async fn session_user(state: &AppState, headers: &HeaderMap) -> Option<SessionUser> {
    let pool = state.db.as_ref()?;
    let token = session_token(headers)?;
    sqlx::query_as::<_, (i64, String, String, String, bool, bool)>(
        r#"SELECT u.id, u.email, u.display_name, u.avatar, u.is_owner, s.unlimited
           FROM sessions s JOIN users u ON u.id = s.user_id
           WHERE s.id = $1 AND s.expires_at > now()"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(
        |(id, email, display_name, avatar, is_owner, unlimited)| SessionUser {
            id,
            email,
            display_name,
            avatar,
            is_owner,
            unlimited,
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
        "acg_session={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=1209600{secure}"
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
    let db = match &state.db {
        Some(pool) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
        None => false,
    };
    let redis_url = if state.cfg.redis_ready() {
        format!("{}/ping", state.cfg.redis_url.trim_end_matches('/'))
    } else {
        String::new()
    };
    let vector_url = if state.cfg.vector_ready() {
        format!("{}/info", state.cfg.vector_url.trim_end_matches('/'))
    } else {
        String::new()
    };
    let r2_url = if state.cfg.r2_ready() {
        state.cfg.r2_public_base.clone()
    } else {
        String::new()
    };
    let (redis, vector, r2) = tokio::join!(
        crate::jobs::probe(&state, &redis_url, &state.cfg.redis_token, false),
        crate::jobs::probe(&state, &vector_url, &state.cfg.vector_token, false),
        crate::jobs::probe(&state, &r2_url, "", true),
    );
    let ready = db && redis != "down" && vector != "down" && r2 != "down";
    Json(json!({
        "status": if ready { "ok" } else { "degraded" },
        "uid": state.cfg.bilibili_uid,
        "db": if db { "up" } else { "down" },
        "redis": redis,
        "r2": r2,
        "vector": vector,
    }))
}

#[derive(Deserialize)]
struct AuthBody {
    email: String,
    password: String,
}

async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> Json<Value> {
    let Some(user) = session_user(&state, &headers).await else {
        return Json(json!({"loggedIn": false}));
    };
    Json(json!({
        "loggedIn": true,
        "unlimited": user.unlimited || user.is_owner,
        "user": user_json(&user),
    }))
}

async fn auth_register(State(state): State<AppState>, Json(body): Json<AuthBody>) -> Response {
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') || body.password.len() < 8 {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_INPUT",
            "请填写有效邮箱，密码至少 8 位",
        );
    }
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
    let owner = email == state.cfg.owner_email && !email.is_empty();
    let inserted = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO users (email, password_hash, is_owner) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&email)
    .bind(&hash)
    .bind(owner)
    .fetch_one(pool)
    .await;
    let Ok((id,)) = inserted else {
        return fail(StatusCode::CONFLICT, "EMAIL_TAKEN", "这个邮箱已经注册过了");
    };
    if owner && !state.cfg.owner_security_answer.is_empty() {
        return begin_owner_challenge(&state, pool, id, &email, &email, "").await;
    }
    issue_session(&state, pool, id, owner).await
}

async fn auth_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthBody>,
) -> Response {
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let email = body.email.trim().to_lowercase();
    let row = sqlx::query_as::<_, (i64, String, String, String, bool)>(
        "SELECT id, password_hash, display_name, avatar, is_owner FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await;
    let Ok(Some((id, hash, display_name, avatar, owner))) = row else {
        return fail(
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "邮箱或密码不正确",
        );
    };
    if bcrypt::verify(&body.password, &hash).ok() != Some(true) {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "auth.login",
                outcome: "failure",
                actor: id,
                target_type: "user",
                target_id: &id.to_string(),
                detail: "bad_password",
            },
        )
        .await
        {
            return *response;
        }
        return fail(
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "邮箱或密码不正确",
        );
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "auth.login",
            outcome: "success",
            actor: id,
            target_type: "user",
            target_id: &id.to_string(),
            detail: "ok",
        },
    )
    .await
    {
        return *response;
    }
    if owner && !state.cfg.owner_security_answer.is_empty() {
        return begin_owner_challenge(&state, pool, id, &email, &display_name, &avatar).await;
    }
    issue_session(&state, pool, id, owner).await
}

async fn begin_owner_challenge(
    state: &AppState,
    pool: &sqlx::PgPool,
    id: i64,
    email: &str,
    display_name: &str,
    avatar: &str,
) -> Response {
    let token = hex::encode(rand::random::<[u8; 16]>());
    let saved = sqlx::query("INSERT INTO login_challenges (token, user_id, expires_at) VALUES ($1, $2, now() + interval '10 minutes')")
        .bind(&token)
        .bind(id)
        .execute(pool)
        .await;
    if saved.is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "安全验证没有准备好",
        );
    }
    Json(json!({
        "ok": true,
        "needsSecurityQuestion": true,
        "securityQuestion": state.cfg.owner_security_question,
        "challengeToken": token,
        "user": {"id": id, "email": email, "displayName": display_name, "avatar": avatar, "isOwner": true}
    })).into_response()
}

async fn issue_session(
    state: &AppState,
    pool: &sqlx::PgPool,
    user_id: i64,
    owner: bool,
) -> Response {
    let token = hex::encode(rand::random::<[u8; 32]>());
    let saved = sqlx::query("INSERT INTO sessions (id, user_id, unlimited, expires_at) VALUES ($1, $2, $3, now() + interval '14 days')")
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
        session_cookie(&token, state.cfg.cookie_secure),
    );
    response
}

async fn auth_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let (Some(pool), Some(token)) = (state.db.as_ref(), session_token(&headers)) {
        if let Err(error) = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(token)
            .execute(pool)
            .await
        {
            let mut response = fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                &format!("会话没有注销: {error}"),
            )
            .into_response();
            response
                .headers_mut()
                .insert(header::SET_COOKIE, clear_cookie(state.cfg.cookie_secure));
            return response;
        }
    }
    let mut response = Json(json!({"ok": true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_cookie(state.cfg.cookie_secure));
    response
}

#[derive(Deserialize)]
struct ProfileBody {
    #[serde(rename = "displayName")]
    display_name: String,
}

async fn auth_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProfileBody>,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    let name = body.display_name.trim();
    if name.is_empty() || name.chars().count() > 24 {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_DISPLAY_NAME",
            "昵称需要 1 到 24 个字",
        );
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
    let mut updated = user;
    updated.display_name = name.to_string();
    Json(json!({"ok": true, "user": user_json(&updated)})).into_response()
}

#[derive(Deserialize)]
struct VerifyBody {
    #[serde(rename = "challengeToken")]
    challenge_token: String,
    answer: String,
}

async fn auth_verify_security(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyBody>,
) -> Response {
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    if state.cfg.owner_security_answer.is_empty()
        || body.answer.trim() != state.cfg.owner_security_answer
    {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "auth.verify",
                outcome: "failure",
                actor: 0,
                target_type: "login_challenge",
                target_id: "",
                detail: "bad_answer",
            },
        )
        .await
        {
            return *response;
        }
        return fail(
            StatusCode::FORBIDDEN,
            "SECURITY_ANSWER_REJECTED",
            "安全答案不正确",
        );
    }
    let user_id = sqlx::query_scalar::<_, i64>(
        "DELETE FROM login_challenges WHERE token = $1 AND expires_at > now() RETURNING user_id",
    )
    .bind(body.challenge_token.trim())
    .fetch_optional(pool)
    .await;
    let user_id = match user_id {
        Ok(value) => value,
        Err(error) => return query_failed(&error),
    };
    let Some(user_id) = user_id else {
        if let Err(response) = note_audit(
            &state,
            &headers,
            AuditNote {
                event: "auth.verify",
                outcome: "failure",
                actor: 0,
                target_type: "login_challenge",
                target_id: "",
                detail: "expired",
            },
        )
        .await
        {
            return *response;
        }
        return fail(
            StatusCode::UNAUTHORIZED,
            "CHALLENGE_EXPIRED",
            "安全验证已过期，请重新登录",
        );
    };
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "auth.verify",
            outcome: "success",
            actor: user_id,
            target_type: "user",
            target_id: &user_id.to_string(),
            detail: "ok",
        },
    )
    .await
    {
        return *response;
    }
    issue_session(&state, pool, user_id, true).await
}

async fn auth_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    let (public, key, _name, _size) = match store_upload(&state, multipart, "avatars").await {
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
    Json(json!({"ok": true, "user": {"id": user.id, "email": user.email, "displayName": user.display_name, "avatar": public, "isOwner": user.is_owner}, "item": {"url": public, "path": key}})).into_response()
}

fn user_json(user: &SessionUser) -> Value {
    json!({
        "id": user.id,
        "email": user.email,
        "displayName": if user.display_name.is_empty() { user.email.clone() } else { user.display_name.clone() },
        "avatar": user.avatar,
        "isOwner": user.is_owner,
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
    content: String,
    channel: Option<String>,
    #[serde(rename = "contactEmail")]
    contact_email: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: Option<i64>,
}

async fn guestbook_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GuestbookBody>,
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
    if parent_id < 0 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_PARENT", "回复楼层不正确");
    }
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
    let user = session_user(&state, &headers).await;
    let admin = user.as_ref().is_some_and(|item| item.is_owner);
    if parent_id > 0 {
        let visible = if admin {
            "status IN ('visible', 'hidden')"
        } else {
            "status = 'visible'"
        };
        let count: i64 = match sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM guestbook_messages WHERE id = $1 AND channel = $2 AND {visible}"
        ))
        .bind(parent_id)
        .bind(&channel)
        .fetch_one(pool)
        .await
        {
            Ok(count) => count,
            Err(_) => {
                return fail(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SERVER_ERROR",
                    "回复楼层没有核对成功",
                )
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
    let region = match crate::guestbook::lookup_region(Some(pool), &state.http, &ip).await {
        Ok(region) => region,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "地区没有查到",
            )
        }
    };
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>)>(
        "INSERT INTO guestbook_messages (user_id, nickname, avatar, channel, content, contact_email, content_hash, ip_hash, ip_region, ip_masked, user_agent_hash, parent_id, is_login_user, is_admin_user) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING id, created_at",
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
    let created_at = created.to_rfc3339();
    let notice = crate::mail::GuestbookNotice {
        channel: &channel,
        nickname: &nickname,
        content,
        created_at: &created_at,
        reply: parent_id > 0,
    };
    if let Err(error) = crate::mail::notify_owner(&state.cfg, &notice).await {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "MAIL_FAILED",
            &format!("留言已保存，通知邮件没有发出: {error}"),
        );
    }
    if admin && parent_id > 0 {
        if let Err(error) = crate::mail::notify_reply(&state.cfg, pool, parent_id, &notice).await {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "MAIL_FAILED",
                &format!("留言已保存，回复通知没有发出: {error}"),
            );
        }
    }
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
    };
    let Some(item) = crate::guestbook::thread_json(vec![item], Vec::new(), false)
        .into_iter()
        .next()
    else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "小纸条没有贴上，请稍后再试哦",
        );
    };
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
        Err(_) => {
            return Err(Box::new(fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "留言频率没有核对成功",
            )))
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
        Err(_) => Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "重复留言没有核对成功",
        ))),
    }
}

async fn guestbook_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    if !user.is_owner {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "只有站长可以删除留言");
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
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                &format!("留言没有删除: {error}"),
            )
        }
    };
    if updated.rows_affected() == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在。");
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
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Response {
    let Some(user) = session_user(&state, &headers).await else {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    };
    if !user.is_owner {
        return fail(StatusCode::FORBIDDEN, "FORBIDDEN", "只有站长可以修改留言");
    }
    let Ok(pool) = db_required(&state) else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "数据库还没有连上",
        );
    };
    let status = body.get("status").and_then(Value::as_str).unwrap_or("");
    if !matches!(status, "visible" | "hidden" | "deleted") {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_STATUS",
            "留言状态只能是可见、隐藏或删除。",
        );
    }
    let updated = match sqlx::query("UPDATE guestbook_messages SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(updated) => updated,
        Err(error) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                &format!("留言状态没有更新: {error}"),
            )
        }
    };
    if updated.rows_affected() == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "留言不存在。");
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
    match list_simple(&state, "SELECT id, title, watched, total, latest_episode, cover_path, link_url FROM bangumi_items ORDER BY updated_at DESC").await {
        Ok(items) => Json(json!({"items": items})).into_response(),
        Err(error) => query_failed(&error),
    }
}

async fn radar_feed(State(state): State<AppState>) -> Response {
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    let rows = match sqlx::query_as::<_, (String, String, String, String, String, bool)>(
        "SELECT id, creator_name, latest_text, cover_path, link_url, is_new FROM radar_feed ORDER BY published_at DESC LIMIT 40",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let items: Vec<Value> = rows.into_iter().map(|(id, name, text, cover, link, is_new)| json!({"id": id, "creatorName": name, "latestText": text, "cover": cover, "linkUrl": link, "isNew": is_new})).collect();
    Json(json!({"items": items})).into_response()
}

async fn list_simple(state: &AppState, sql: &str) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (String, String, i32, i32, i32, String, String)>(sql)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(id, title, watched, total, latest, cover, link)| json!({"id": id, "title": title, "watched": watched, "total": total, "latestEpisode": latest, "cover": cover, "linkUrl": link})).collect())
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
                (header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg")),
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

#[derive(Deserialize)]
struct WallpaperQuery {
    source: Option<String>,
    #[serde(rename = "apiOnly")]
    api_only: Option<String>,
}

fn wants_api_wallpaper(source: Option<&str>, api_only: Option<&str>) -> bool {
    source == Some("api") || api_only == Some("1")
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
            "addedAt": Utc::now().to_rfc3339()
        })
    } else {
        json!({
            "id": "fallback-archive",
            "url": "https://images.taozhiyy.top/wallpapers/fallback.jpg",
            "previewUrl": "https://images.taozhiyy.top/wallpapers/fallback.jpg",
            "source": "r2",
            "author": "taozhiyy",
            "sourceUrl": "https://taozhiyy.top/",
            "licenseNote": "站点图池为空时的占位",
            "kind": "fallback",
            "addedAt": Utc::now().to_rfc3339()
        })
    };
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"item": fallback, "fallback": true, "apiOnly": api_only}))
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
                return Json(json!({"item": fallback, "fallback": true, "apiOnly": true, "syncError": error.to_string()}))
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
        return Json(json!({"item": fallback, "fallback": true, "apiOnly": api_only}))
            .into_response();
    };
    Json(json!({"item": {"id": id, "url": url, "previewUrl": preview, "source": source, "author": author, "sourceUrl": source_url, "licenseNote": license, "kind": kind, "addedAt": added.to_rfc3339()}, "apiOnly": api_only})).into_response()
}

async fn search(State(state): State<AppState>, Query(query): Query<SearchQuery>) -> Response {
    if !state.cfg.vector_ready() {
        return Json(json!({"configured": false, "items": []})).into_response();
    }
    let response = match state.http.post(format!("{}/query", state.cfg.vector_url.trim_end_matches('/')))
        .bearer_auth(&state.cfg.vector_token)
        .json(&json!({"data": query.q, "topK": query.limit.unwrap_or(10), "includeMetadata": true, "includeData": false}))
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => return fail(StatusCode::BAD_GATEWAY, "SEARCH_UPSTREAM", &error.to_string()),
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
            return fail(
                StatusCode::BAD_GATEWAY,
                "SEARCH_UPSTREAM",
                &error.to_string(),
            )
        }
    };
    match crate::search_hits::hits_from_vector(&body) {
        Ok(items) => Json(json!({"configured": true, "items": items})).into_response(),
        Err(error) => fail(
            StatusCode::BAD_GATEWAY,
            "SEARCH_UPSTREAM",
            &error.to_string(),
        ),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<u32>,
}

async fn sync_trigger(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = authorize_sync_trigger(&state, &headers).await {
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
    if user.as_ref().is_some_and(|item| item.is_owner) {
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
    let owner = session_user(&state, &headers)
        .await
        .is_some_and(|user| user.is_owner);
    if !owner {
        if let Err(response) = scheduler_authorized(&state, &headers, &uri, &body) {
            return *response;
        }
    }
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
    match crate::wallpaper::sync_pool(&state).await {
        Ok(count) => Json(json!({"ok": true, "count": count})).into_response(),
        Err(error) => fail(StatusCode::BAD_GATEWAY, "SYNC_FAILED", &error.to_string()),
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
    match crate::feed::sync_radar(&state, Utc::now()).await {
        Ok(report) => Json(json!({"ok": true, "saved": report.saved, "failed": report.failed}))
            .into_response(),
        Err(error) => fail(StatusCode::BAD_GATEWAY, "SYNC_FAILED", &error.to_string()),
    }
}

async fn chat_quota(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = session_user(&state, &headers).await;
    let (limit, used) = match quota_of(&state, &headers, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    Json(quota_json(&state, &user, limit, used)).into_response()
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
    Json(body): Json<ChatBody>,
) -> Response {
    if let Err(response) = admit(&state, &headers, &state.chat_gate, "chat", 20).await {
        return *response;
    }
    let user = session_user(&state, &headers).await;
    let (limit, used) = match quota_of(&state, &headers, &user).await {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let mut quota = quota_json(&state, &user, limit, used);
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
        quota["chatEnabled"] = json!(false);
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
    let endpoint = format!(
        "{}/v1/chat/completions",
        state.cfg.deepseek_base_url.trim_end_matches('/')
    );
    let response = state
        .http
        .post(endpoint)
        .bearer_auth(&state.cfg.deepseek_api_key)
        .timeout(Duration::from_secs(45))
        .json(&crate::chat_prompt::request_body(
            &state.cfg.deepseek_model,
            &prompt,
        ))
        .send()
        .await;
    let response = match response {
        Ok(item) => item,
        Err(error) => {
            if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                return *response;
            }
            return chat_upstream(quota, &error.to_string());
        }
    };
    let response = match response.error_for_status() {
        Ok(item) => item,
        Err(error) => {
            if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                return *response;
            }
            return chat_upstream(quota, &error.to_string());
        }
    };
    let payload = match response.json::<Value>().await {
        Ok(body) => body,
        Err(error) => {
            if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
                return *response;
            }
            return chat_upstream(quota, &error.to_string());
        }
    };
    let Some(reply) = payload["choices"][0]["message"]["content"]
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
    else {
        if let Err(response) = record_chat(&state, &user, ChatStat::UpstreamError).await {
            return *response;
        }
        return chat_upstream(quota, "模型没有返回回复");
    };
    if let Err(response) = record_chat(&state, &user, ChatStat::Success).await {
        return *response;
    }
    if let Err(response) = bump_quota(&state, &headers, &user).await {
        return *response;
    }
    quota["used"] = json!(used + 1);
    quota["remaining"] = json!((limit - used - 1).max(0));
    quota["reply"] = json!(reply);
    Json(quota).into_response()
}

fn quota_json(state: &AppState, user: &Option<SessionUser>, limit: i64, used: i64) -> Value {
    let unlimited = user
        .as_ref()
        .is_some_and(|item| item.unlimited || item.is_owner);
    json!({
        "limit": if unlimited { -1 } else { limit },
        "used": used,
        "remaining": if unlimited { -1 } else { (limit - used).max(0) },
        "isLogin": user.is_some(),
        "unlimited": unlimited,
        "chatEnabled": !state.cfg.deepseek_api_key.is_empty(),
        "deepseekEnabled": !state.cfg.deepseek_api_key.is_empty(),
        "fixedAnswersEnabled": true,
        "imageEnabled": !state.cfg.dashscope_api_key.is_empty(),
    })
}

fn chat_daily_limit(user: &Option<SessionUser>) -> i64 {
    if user
        .as_ref()
        .is_some_and(|item| item.unlimited || item.is_owner)
    {
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
    let Some(pool) = state.db.as_ref() else {
        return Ok((limit, 0));
    };
    let used: i64 = sqlx::query_scalar(
        "SELECT used::bigint FROM ai_chat_quota WHERE identity_key = $1 AND quota_date = CURRENT_DATE",
    )
    .bind(chat_quota_key(user, headers))
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
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    let claimed = sqlx::query_as::<_, (i64, bool)>(
        "WITH existing AS (
            SELECT used FROM ai_chat_quota WHERE identity_key = $1 AND quota_date = CURRENT_DATE
         ), updated AS (
            INSERT INTO ai_chat_quota (identity_key, quota_date, used, last_request_at)
            VALUES ($1, CURRENT_DATE, 0, now())
            ON CONFLICT (identity_key, quota_date) DO UPDATE
            SET last_request_at = now()
            WHERE ai_chat_quota.last_request_at IS NULL
               OR ai_chat_quota.last_request_at <= now() - interval '5 seconds'
            RETURNING used
         )
         SELECT used::bigint, true AS claimed FROM updated
         UNION ALL
         SELECT used::bigint, false AS claimed FROM existing
         WHERE NOT EXISTS (SELECT 1 FROM updated)",
    )
    .bind(chat_quota_key(user, headers))
    .fetch_one(pool)
    .await;
    let (used, open) = match claimed {
        Ok(row) => row,
        Err(error) => return Err(Box::new(query_failed(&error))),
    };
    if open {
        return Ok(());
    }
    record_chat(state, user, ChatStat::RateDenied).await?;
    let mut quota = quota_json(state, user, limit, used);
    quota["error"] = json!("RATE_LIMIT_EXCEEDED");
    quota["message"] = json!("提问太快啦，请稍等 5 秒再试～");
    Err(Box::new(
        (StatusCode::TOO_MANY_REQUESTS, Json(quota)).into_response(),
    ))
}

async fn bump_quota(
    state: &AppState,
    headers: &HeaderMap,
    user: &Option<SessionUser>,
) -> Result<(), Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    let key = chat_quota_key(user, headers);
    let updated = sqlx::query(
        "UPDATE ai_chat_quota SET used = used + 1 WHERE identity_key = $1 AND quota_date = CURRENT_DATE",
    )
    .bind(key)
    .execute(pool)
    .await
    .map_err(|error| Box::new(query_failed(&error)))?;
    if updated.rows_affected() == 0 {
        return Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "额度没有记上",
        )));
    }
    Ok(())
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
    let guest = i32::from(user.is_none());
    let member = i32::from(user.is_some());
    let hour = Utc::now().format("%Y-%m-%dT%H").to_string();
    let day = Utc::now().date_naive();
    sqlx::query(
        "INSERT INTO ai_chat_hourly (bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (bucket) DO UPDATE SET success = ai_chat_hourly.success + EXCLUDED.success, upstream_error = ai_chat_hourly.upstream_error + EXCLUDED.upstream_error,
           quota_denied = ai_chat_hourly.quota_denied + EXCLUDED.quota_denied, rate_denied = ai_chat_hourly.rate_denied + EXCLUDED.rate_denied,
           not_configured = ai_chat_hourly.not_configured + EXCLUDED.not_configured, guest_calls = ai_chat_hourly.guest_calls + EXCLUDED.guest_calls, user_calls = ai_chat_hourly.user_calls + EXCLUDED.user_calls",
    )
    .bind(hour).bind(success).bind(upstream).bind(quota).bind(rate).bind(missing).bind(guest).bind(member)
    .execute(pool).await.map_err(|error| Box::new(query_failed(&error)))?;
    sqlx::query(
        "INSERT INTO ai_chat_daily (bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (bucket) DO UPDATE SET success = ai_chat_daily.success + EXCLUDED.success, upstream_error = ai_chat_daily.upstream_error + EXCLUDED.upstream_error,
           quota_denied = ai_chat_daily.quota_denied + EXCLUDED.quota_denied, rate_denied = ai_chat_daily.rate_denied + EXCLUDED.rate_denied,
           not_configured = ai_chat_daily.not_configured + EXCLUDED.not_configured, guest_calls = ai_chat_daily.guest_calls + EXCLUDED.guest_calls, user_calls = ai_chat_daily.user_calls + EXCLUDED.user_calls",
    )
    .bind(day).bind(success).bind(upstream).bind(quota).bind(rate).bind(missing).bind(guest).bind(member)
    .execute(pool).await.map_err(|error| Box::new(query_failed(&error)))?;
    Ok(())
}

async fn fixed_answer(
    state: &AppState,
    message: &str,
) -> Result<Option<(i64, String)>, Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(None);
    };
    sqlx::query_as(
        "SELECT id, answer FROM ai_fixed_answers WHERE status = 'active' AND lower(question) = lower($1) LIMIT 1",
    )
    .bind(message)
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
    days: Option<i64>,
}

async fn chat_stats(State(state): State<AppState>, Query(query): Query<DaysQuery>) -> Response {
    let days = query.days.unwrap_or(14).clamp(1, 90);
    let today = Utc::now().date_naive();
    let since = today - chrono::Duration::days(days - 1);
    let (daily, hourly) = if let Some(pool) = state.db.as_ref() {
        let daily = match sqlx::query_as::<_, (NaiveDate, i32, i32, i32, i32, i32, i32, i32)>(
            "SELECT bucket, success, upstream_error, quota_denied, rate_denied, not_configured, guest_calls, user_calls FROM ai_chat_daily WHERE bucket >= $1 ORDER BY bucket",
        )
        .bind(since)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(error) => return query_failed(&error),
        };
        let prefix = Utc::now().format("%Y-%m-%dT").to_string();
        let hourly = match sqlx::query_as::<_, (String, i32, i32, i32, i32, i32)>(
            "SELECT bucket, success, upstream_error, quota_denied, rate_denied, not_configured FROM ai_chat_hourly WHERE bucket LIKE $1 || '%' ORDER BY bucket",
        )
        .bind(prefix)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(error) => return query_failed(&error),
        };
        (daily, hourly)
    } else {
        (Vec::new(), Vec::new())
    };
    Json(chat_stats_json(
        &state.cfg.deepseek_model,
        !state.cfg.deepseek_api_key.is_empty(),
        today,
        &daily,
        &hourly,
    ))
    .into_response()
}

type DailyRow = (NaiveDate, i32, i32, i32, i32, i32, i32, i32);
type HourlyRow = (String, i32, i32, i32, i32, i32);

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
        json!({"hour": format!("{hour:02}"), "success": success, "upstreamError": upstream, "quotaDenied": quota, "rateDenied": rate, "notConfigured": missing, "total": success + upstream + quota + rate + missing})
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
    let enabled = !state.cfg.dashscope_api_key.is_empty();
    json!({
        "limit": if unlimited { -1 } else { limit },
        "used": used,
        "remaining": if unlimited { -1 } else { (limit - used).max(0) },
        "isLogin": user.is_some(),
        "unlimited": unlimited,
        "imageEnabled": enabled,
        "canGenerate": user.is_some() && enabled && (unlimited || used < limit),
        "model": "wanx-v1",
        "promptExtend": false,
        "userId": user.as_ref().map(|item| item.id).unwrap_or(0)
    })
}

async fn image_quota_of(
    state: &AppState,
    user: &Option<SessionUser>,
) -> Result<(i64, i64), Box<Response>> {
    const LIMIT: i64 = 3;
    let Some(pool) = state.db.as_ref() else {
        return Ok((LIMIT, 0));
    };
    let Some(user) = user else {
        return Ok((LIMIT, 0));
    };
    let used: i64 = sqlx::query_scalar(
        "SELECT used::bigint FROM ai_image_quota WHERE identity_key = $1 AND quota_date = $2",
    )
    .bind(format!("user:{}", user.id))
    .bind(Utc::now().date_naive())
    .fetch_optional(pool)
    .await
    .map_err(|error| Box::new(query_failed(&error)))?
    .unwrap_or(0);
    Ok((LIMIT, used))
}

async fn bump_image_quota(state: &AppState, user: &SessionUser) -> Result<(), Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Ok(());
    };
    sqlx::query("INSERT INTO ai_image_quota (identity_key, quota_date, used) VALUES ($1, $2, 1) ON CONFLICT (identity_key, quota_date) DO UPDATE SET used = ai_image_quota.used + 1")
        .bind(format!("user:{}", user.id))
        .bind(Utc::now().date_naive())
        .execute(pool)
        .await
        .map_err(|error| Box::new(query_failed(&error)))?;
    Ok(())
}

#[derive(Deserialize)]
struct ImageBody {
    prompt: String,
    size: Option<String>,
}

async fn image_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ImageBody>,
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
    if user.is_none() {
        let mut body = quota;
        body["error"] = json!("LOGIN_REQUIRED");
        body["message"] = json!("请先登录后再生图，避免额度被刷爆。");
        return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
    }
    if state.cfg.dashscope_api_key.is_empty() {
        let mut body = quota;
        body["error"] = json!("IMAGE_NOT_CONFIGURED");
        body["message"] = json!("生图还没配置 Agnes API Key，站长配好后就能用了。");
        return (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response();
    }
    let prompt = body.prompt.trim();
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
    let unlimited = user
        .as_ref()
        .is_some_and(|item| item.unlimited || item.is_owner);
    if !unlimited && used >= limit {
        let mut body = quota;
        body["error"] = json!("DAILY_LIMIT_EXCEEDED");
        body["message"] = json!("今日生图次数用完啦，明天再来画吧～");
        return (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
    }
    let size = body.size.unwrap_or_else(|| "1024*1024".into());
    let response = state
        .http
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis")
        .bearer_auth(&state.cfg.dashscope_api_key)
        .json(
            &json!({"model": "wanx-v1", "input": {"prompt": prompt}, "parameters": {"size": size}}),
        )
        .send()
        .await;
    let Ok(response) = response else {
        return fail(
            StatusCode::BAD_GATEWAY,
            "UPSTREAM_ERROR",
            "生图服务没有回应",
        );
    };
    let Ok(payload) = response.json::<Value>().await else {
        return fail(
            StatusCode::BAD_GATEWAY,
            "UPSTREAM_ERROR",
            "生图结果无法解析",
        );
    };
    let url = payload["output"]["results"][0]["url"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if url.is_empty() {
        return fail(
            StatusCode::BAD_GATEWAY,
            "UPSTREAM_ERROR",
            "生图没有返回图片",
        );
    }
    let stored = match crate::image_store::store_remote_image(&state, &url).await {
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
            return fail(status, code, &error.to_string());
        }
    };
    let Some(pool) = state.db.as_ref() else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_UNAVAILABLE",
            "生成图已保存到 R2，但数据库还没有连上",
        );
    };
    let author = user
        .as_ref()
        .map(|item| item.display_name.as_str())
        .unwrap_or("");
    if let Err(error) = sqlx::query(
        "INSERT INTO ai_images (prompt, image_url, size, author, object_key) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(prompt)
    .bind(&stored.url)
    .bind(&size)
    .bind(author)
    .bind(&stored.key)
    .execute(pool)
    .await
    {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", &format!("生成图已保存到 R2，记录没有写上: {error}"));
    }
    if let Some(current) = user.as_ref().filter(|_| !unlimited) {
        if let Err(response) = bump_image_quota(&state, current).await {
            return *response;
        }
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
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
    let used = if unlimited { used } else { used + 1 };
    let mut out = image_quota_json(&state, &user, limit, used);
    out["image"] = json!({"url": stored.url, "path": stored.key, "mimeType": stored.mime, "size": stored.size, "width": 0, "height": 0});
    out["size"] = json!(size);
    Json(out).into_response()
}

#[derive(Deserialize)]
struct GalleryQuery {
    limit: Option<i64>,
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
    let limit = query.limit.unwrap_or(60).clamp(1, 200);
    let before = query
        .before
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc));
    let rows = sqlx::query_as::<_, (String, String, String, String, DateTime<Utc>)>(
        "SELECT prompt, image_url, size, author, created_at FROM ai_images WHERE ($1::timestamptz IS NULL OR created_at < $1) ORDER BY created_at DESC LIMIT $2",
    )
    .bind(before)
    .bind(limit)
    .fetch_all(pool)
    .await;
    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                &error.to_string(),
            )
        }
    };
    let full = rows.len() as i64 == limit;
    let items: Vec<Value> = rows.into_iter().map(|(prompt, url, size, author, created)| json!({"prompt": prompt, "imageUrl": url, "size": size, "author": author, "createdAt": created.to_rfc3339()})).collect();
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
        "runtime": "rust",
        "cpuCores": host.cpu_cores,
        "goroutines": host.cpu_cores,
        "memory": {"usedMB": host.memory_used_mb, "totalMB": host.memory_total_mb, "usedRatio": host.memory_ratio, "source": host.memory_source},
        "cpuPercent": host.cpu_percent,
        "uptime": format_uptime(state.started.elapsed()),
        "uptimeSecs": state.started.elapsed().as_secs(),
        "status": "online",
        "serverTime": Utc::now().to_rfc3339()
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
    let region = match crate::guestbook::lookup_region(state.db.as_ref(), &state.http, &ip).await {
        Ok(region) => region,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVER_ERROR",
                "地区没有查到",
            )
        }
    };
    Json(json!({
        "regionLabel": crate::guestbook::compact_region_label(&region),
        "ipRegion": region,
        "ipMasked": crate::guestbook::mask_ip(&ip),
        "serverTime": Utc::now().to_rfc3339()
    }))
    .into_response()
}

fn client_ip(headers: &HeaderMap) -> String {
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
        Some(user) if user.is_owner => Ok(user),
        Some(_) => Err(Box::new(fail(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "只有站长可以操作",
        ))),
        None => Err(Box::new(fail(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "请先登录",
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
        match sqlx::query_scalar("SELECT success::bigint FROM ai_chat_daily WHERE bucket = $1")
            .bind(Utc::now().date_naive())
            .fetch_optional(pool)
            .await
        {
            Ok(value) => value.unwrap_or(0),
            Err(error) => return query_failed(&error),
        }
    } else {
        0
    };
    Json(json!({
        "owner": {"id": user.id, "email": user.email, "displayName": user.display_name},
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
            let display = if name.trim().is_empty() { email.clone() } else { name };
            json!({"id": id, "email": email, "displayName": display, "createdAt": created.to_rfc3339()})
        }).collect::<Vec<_>>(),
        "guestbookContacts": contacts.into_iter().filter_map(|(id, channel, nickname, content, submitted, account, created)| {
            let submitted = submitted.trim().to_lowercase();
            let account = account.trim().to_lowercase();
            let contact = if submitted.is_empty() { account.clone() } else { submitted };
            if contact.is_empty() {
                return None;
            }
            Some(json!({"id": id, "source": channel, "nickname": nickname, "content": content, "contactEmail": contact, "accountEmail": account, "createdAt": created.to_rfc3339()}))
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
    Json(body): Json<Value>,
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
        .unwrap_or("article");
    let text = body.get("body").and_then(Value::as_str).unwrap_or("");
    if text.len() > 200_000 {
        return fail(StatusCode::BAD_REQUEST, "INVALID_BODY", "草稿正文太大。");
    }
    let cover = body.get("coverUrl").and_then(Value::as_str).unwrap_or("");
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft");
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
    Json(json!({"ok": true, "item": {"id": id, "kind": kind, "title": title, "body": text, "coverUrl": cover, "status": status, "createdAt": created.to_rfc3339(), "updatedAt": updated.to_rfc3339()}})).into_response()
}

fn owner_payload(user: &SessionUser, item: Value) -> Value {
    json!({"ok": true, "item": item, "owner": {"email": user.email, "displayName": user.display_name}})
}

async fn owner_publish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let spec = match publish_spec(&state) {
        Ok(spec) => spec,
        Err(response) => {
            return publish_not_configured(
                &state,
                &headers,
                owner.id,
                "owner.publish",
                "article",
                response,
            )
            .await
        }
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
            return fail(
                StatusCode::BAD_REQUEST,
                "INVALID_PUBLISH",
                &error.to_string(),
            )
        }
    };
    let published = match crate::github_publish::publish_article(
        &state.http,
        &spec,
        title,
        &article.markdown,
        &article.path,
        now,
    )
    .await
    {
        Ok(published) => published,
        Err(error) => {
            return publish_error(
                &state,
                &headers,
                PublishFailure {
                    actor: owner.id,
                    event: "owner.publish",
                    target: &article.path,
                    invalid_code: "INVALID_PUBLISH",
                    github_code: "PUBLISH_FAILED",
                },
                error,
            )
            .await
        }
    };
    if draft_id > 0 {
        if let Err(response) = mark_draft_published(&state, draft_id).await {
            return *response;
        }
    }
    if let Err(response) = note_audit(
        &state,
        &headers,
        AuditNote {
            event: "owner.publish",
            outcome: "success",
            actor: owner.id,
            target_type: "owner_publish",
            target_id: &published.path,
            detail: "published",
        },
    )
    .await
    {
        return *response;
    }
    Json(owner_payload(&owner, json!({
        "path": published.path,
        "commitSha": published.commit_sha,
        "branch": published.branch,
        "commitMessage": published.commit_message,
        "repoURL": format!("https://github.com/{}/{}/blob/{}/{}", spec.owner, spec.repo, published.branch, published.path)
    }))).into_response()
}

async fn mark_draft_published(state: &AppState, draft_id: i64) -> Result<(), Box<Response>> {
    let Some(pool) = state.db.as_ref() else {
        return Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DRAFT_UPDATE_FAILED",
            "文章已写入仓库，但数据库还没有连上，草稿状态没有更新",
        )));
    };
    let updated = sqlx::query("UPDATE owner_drafts SET status = 'published' WHERE id = $1")
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
        return Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DRAFT_UPDATE_FAILED",
            "文章已写入仓库，但草稿不存在",
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
struct FriendBody {
    name: String,
    desc: String,
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
    #[serde(rename = "imageUrl")]
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
        crate::github_publish::PublishError::GitHub { message, .. }
        | crate::github_publish::PublishError::Transport(message)
        | crate::github_publish::PublishError::Response(message) => {
            fail(StatusCode::BAD_GATEWAY, failure.github_code, &message)
        }
    }
}

async fn owner_gallery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GalleryBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let spec = match publish_spec(&state) {
        Ok(spec) => spec,
        Err(response) => {
            return publish_not_configured(
                &state,
                &headers,
                owner.id,
                "owner.gallery_publish",
                crate::github_publish::GALLERY_PATH,
                response,
            )
            .await
        }
    };
    let input = crate::js_source::GalleryInput {
        album_id: &body.album_id,
        album_title: &body.album_title,
        image_url: &body.image_url,
    };
    match crate::github_publish::publish_gallery(&state.http, &spec, input, Utc::now()).await {
        Ok(item) => {
            if let Err(response) = note_audit(
                &state,
                &headers,
                AuditNote {
                    event: "owner.gallery_publish",
                    outcome: "success",
                    actor: owner.id,
                    target_type: "owner_publish",
                    target_id: &item.path,
                    detail: "published",
                },
            )
            .await
            {
                return *response;
            }
            Json(owner_payload(&owner, json!({
            "albumId": item.album_id,
            "imageUrl": item.image_url,
            "path": item.path,
            "commitSha": item.commit_sha,
            "changed": item.changed,
            "branch": item.branch,
            "commitMessage": item.commit_message,
            "repoURL": format!("https://github.com/{}/{}/blob/{}/{}", spec.owner, spec.repo, item.branch, item.path)
        }))).into_response()
        }
        Err(error) => {
            publish_error(
                &state,
                &headers,
                PublishFailure {
                    actor: owner.id,
                    event: "owner.gallery_publish",
                    target: crate::github_publish::GALLERY_PATH,
                    invalid_code: "INVALID_GALLERY_PUBLISH",
                    github_code: "GALLERY_PUBLISH_FAILED",
                },
                error,
            )
            .await
        }
    }
}

async fn owner_friends(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FriendBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let spec = match publish_spec(&state) {
        Ok(spec) => spec,
        Err(response) => {
            return publish_not_configured(
                &state,
                &headers,
                owner.id,
                "owner.friend_publish",
                crate::github_publish::FRIEND_CARDS_PATH,
                response,
            )
            .await
        }
    };
    let input = crate::js_source::FriendInput {
        name: &body.name,
        desc: &body.desc,
        url: &body.url,
        avatar: &body.avatar,
    };
    match crate::github_publish::publish_friend(&state.http, &spec, input, Utc::now()).await {
        Ok(item) => {
            if let Err(response) = note_audit(
                &state,
                &headers,
                AuditNote {
                    event: "owner.friend_publish",
                    outcome: "success",
                    actor: owner.id,
                    target_type: "owner_publish",
                    target_id: &item.path,
                    detail: "published",
                },
            )
            .await
            {
                return *response;
            }
            Json(owner_payload(&owner, json!({
            "name": item.name,
            "url": item.url,
            "path": item.path,
            "commitSha": item.commit_sha,
            "changed": item.changed,
            "branch": item.branch,
            "commitMessage": item.commit_message,
            "pullRequestURL": item.pull_request_url,
            "pullRequestNumber": item.pull_request_number,
            "repoURL": format!("https://github.com/{}/{}/blob/{}/{}", spec.owner, spec.repo, item.branch, item.path)
        }))).into_response()
        }
        Err(error) => {
            publish_error(
                &state,
                &headers,
                PublishFailure {
                    actor: owner.id,
                    event: "owner.friend_publish",
                    target: crate::github_publish::FRIEND_CARDS_PATH,
                    invalid_code: "INVALID_FRIEND_PUBLISH",
                    github_code: "FRIEND_PUBLISH_FAILED",
                },
                error,
            )
            .await
        }
    }
}

async fn owner_moments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MomentBody>,
) -> Response {
    let owner = match require_owner(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return *response,
    };
    let spec = match publish_spec(&state) {
        Ok(spec) => spec,
        Err(response) => {
            return publish_not_configured(
                &state,
                &headers,
                owner.id,
                "owner.moment_publish",
                crate::github_publish::MOMENTS_PATH,
                response,
            )
            .await
        }
    };
    let input = crate::js_source::MomentInput {
        year: &body.year,
        date: &body.date,
        kind: &body.kind,
        category: &body.category,
        content: &body.content,
    };
    match crate::github_publish::publish_moment(&state.http, &spec, input, Utc::now()).await {
        Ok(item) => {
            if let Err(response) = note_audit(
                &state,
                &headers,
                AuditNote {
                    event: "owner.moment_publish",
                    outcome: "success",
                    actor: owner.id,
                    target_type: "owner_publish",
                    target_id: &item.path,
                    detail: "published",
                },
            )
            .await
            {
                return *response;
            }
            Json(owner_payload(&owner, json!({
            "year": item.year,
            "date": item.date,
            "type": item.kind,
            "path": item.path,
            "commitSha": item.commit_sha,
            "changed": item.changed,
            "branch": item.branch,
            "commitMessage": item.commit_message,
            "repoURL": format!("https://github.com/{}/{}/blob/{}/{}", spec.owner, spec.repo, item.branch, item.path)
        }))).into_response()
        }
        Err(error) => {
            publish_error(
                &state,
                &headers,
                PublishFailure {
                    actor: owner.id,
                    event: "owner.moment_publish",
                    target: crate::github_publish::MOMENTS_PATH,
                    invalid_code: "INVALID_MOMENT_PUBLISH",
                    github_code: "MOMENT_PUBLISH_FAILED",
                },
                error,
            )
            .await
        }
    }
}

async fn fixed_answers(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if require_owner(&state, &headers).await.is_err() {
        return fail(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录");
    }
    let Some(pool) = state.db.as_ref() else {
        return Json(json!({"items": []})).into_response();
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, question, answer, status FROM ai_fixed_answers ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await;
    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => return query_failed(&error),
    };
    let items: Vec<Value> = rows.into_iter().map(|(id, question, answer, status)| json!({"id": id, "question": question, "answer": answer, "status": status})).collect();
    Json(json!({"items": items})).into_response()
}

async fn fixed_answer_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
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
    let question = body
        .get("question")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let answer = body
        .get("answer")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if question.is_empty() || answer.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_FIXED_ANSWER",
            "固定答案需要填写问题和答案，且内容不能过长。",
        );
    }
    let row = sqlx::query_as::<_, (i64, DateTime<Utc>)>(
        "INSERT INTO ai_fixed_answers (question, answer) VALUES ($1, $2) RETURNING id, created_at",
    )
    .bind(question)
    .bind(answer)
    .fetch_one(pool)
    .await;
    let Ok((id, created)) = row else {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERVER_ERROR",
            "固定答案没有保存",
        );
    };
    Json(json!({"ok": true, "item": {"id": id, "question": question, "answer": answer, "status": "active", "createdAt": created.to_rfc3339(), "updatedAt": created.to_rfc3339()}})).into_response()
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
    let updated = sqlx::query("UPDATE guestbook_messages SET owner_read_at = now() WHERE id = $1 AND status = 'visible' AND owner_read_at IS NULL")
        .bind(id).execute(pool).await;
    if updated.map(|result| result.rows_affected()).unwrap_or(0) == 0 {
        return fail(StatusCode::NOT_FOUND, "NOT_FOUND", "提醒不存在。");
    }
    Json(json!({"ok": true, "id": id, "ownerReadAt": Utc::now().to_rfc3339()})).into_response()
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
    Json(body): Json<PresignBody>,
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
        Ok((public, key, name, size)) => Json(json!({"ok": true, "item": {"name": name, "url": public, "path": key, "size": size, "mimeType": "image/jpeg"}})).into_response(),
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
    if kind != "gallery" && kind != "article" {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_ASSET_KIND",
            "资源类型必须是相册或文章。",
        );
    }
    if kind == "gallery" && album.trim().is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "INVALID_ALBUM",
            "相册上传必须选择相册。",
        );
    }
    match put_bytes(&state, &format!("{kind}/{album}"), &name, bytes).await {
        Ok((public, key, name, size)) => Json(json!({"ok": true, "item": {"kind": kind, "url": public, "path": key, "name": name, "size": size, "mimeType": "image/jpeg"}})).into_response(),
        Err(response) => *response,
    }
}

async fn store_upload(
    state: &AppState,
    mut multipart: Multipart,
    folder: &str,
) -> Result<(String, String, String, usize), Box<Response>> {
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
    name: &str,
    bytes: Bytes,
) -> Result<(String, String, String, usize), Box<Response>> {
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
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(Box::new(fail(
            StatusCode::BAD_REQUEST,
            "FILE_TOO_LARGE",
            "上传文件超过 8 MiB 限制。",
        )));
    }
    let safe_name = name.replace(['/', '\\'], "-");
    let key = format!("{folder}/{}/{}", Utc::now().format("%Y%m%d"), safe_name);
    let hash = hex::encode(Sha256::digest(&bytes));
    let size = bytes.len();
    let url = signed_put_url(&state.cfg, &key, &hash, 300);
    let sent = state.http.put(url).body(bytes).send().await;
    if sent.and_then(|item| item.error_for_status()).is_err() {
        return Err(Box::new(fail(
            StatusCode::BAD_GATEWAY,
            "R2_ERROR",
            "上传到 R2 失败",
        )));
    }
    Ok((public_url(&state.cfg, &key), key, safe_name, size))
}

async fn owner_draft_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)>("SELECT id, kind, title, body, cover_url, status, created_at, updated_at FROM owner_drafts ORDER BY updated_at DESC, id DESC LIMIT 20")
        .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, kind, title, body, cover, status, created, updated)| json!({"id": id, "kind": kind, "title": title, "body": body, "coverUrl": cover, "status": status, "createdAt": created.to_rfc3339(), "updatedAt": updated.to_rfc3339()})).collect())
}

async fn fixed_answer_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>)>(
        "SELECT id, question, answer, status, created_at FROM ai_fixed_answers ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id, question, answer, status, created)| json!({"id": id, "question": question, "answer": answer, "status": status, "createdAt": created.to_rfc3339(), "updatedAt": created.to_rfc3339()})).collect())
}

async fn notification_items(pool: Option<&sqlx::PgPool>) -> Result<Vec<Value>, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, DateTime<Utc>)>(
        "SELECT id, channel, nickname, content, contact_email, created_at FROM guestbook_messages WHERE status = 'visible' AND owner_read_at IS NULL ORDER BY id DESC LIMIT 30",
    )
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(id, channel, nickname, content, email, created)| {
            json!({
                "id": id,
                "source": channel,
                "title": crate::guestbook::notice_title(&channel),
                "detail": crate::guestbook::notice_detail(&channel, &nickname, created),
                "nickname": nickname,
                "content": content,
                "contactEmail": email.trim().to_lowercase(),
                "createdAt": created.to_rfc3339(),
                "count": 1
            })
        })
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
        let key = format!("rl:{bucket}:{ip}:{}", Utc::now().format("%Y%m%d%H%M"));
        return match crate::rate_limit::allow_fixed_window(
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
            Err(error) => Err(Box::new(fail(
                StatusCode::SERVICE_UNAVAILABLE,
                "RATE_BACKEND",
                &error,
            ))),
        };
    }
    if gate.allow(&ip) {
        Ok(())
    } else {
        Err(Box::new(rate_limited()))
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
    let Some(pool) = state.db.as_ref() else {
        return Err(Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUDIT_FAILED",
            "安全审计没有写上：数据库还没有连上",
        )));
    };
    let user_agent = match headers.get(header::USER_AGENT) {
        None => String::new(),
        Some(value) => value
            .to_str()
            .map(str::to_string)
            .unwrap_or_else(|_| "invalid-user-agent".to_string()),
    };
    let ip = client_ip(headers);
    crate::audit::record(
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
    .map_err(|error| {
        Box::new(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUDIT_FAILED",
            &error.to_string(),
        ))
    })
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
}
