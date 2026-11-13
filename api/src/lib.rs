mod account;
pub mod audit;
mod bili;
pub mod chat_prompt;
pub mod chat_quota;
pub mod config;
mod db;
pub mod feed;
mod fixed_answers;
pub mod github_publish;
pub mod guestbook;
pub mod host;
mod image_gen;
mod image_store;
pub mod jobs;
pub mod js_source;
pub mod mail;
pub mod post_markdown;
pub mod qstash;
mod r2;
pub mod rate_limit;
mod routes;
pub mod search_hits;
pub mod sync_auth;
pub mod wallpaper;
mod wbi;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::PgPool;

pub use config::Config;
use rate_limit::RateGate;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub db: Option<PgPool>,
    pub http: reqwest::Client,
    pub started: Instant,
    pub chat_gate: RateGate,
    pub image_gate: RateGate,
    pub gallery_gate: RateGate,
    /// 注册、登录、学号验证和头像上传共用。15 次/分钟。
    pub auth_gate: RateGate,
    /// 手动同步是否已经在跑。同一时刻只允许一次。
    pub sync_running: Arc<AtomicBool>,
}

impl AppState {
    pub async fn connect(cfg: Config) -> Self {
        let db = if let Some(url) = cfg.database_url.clone() {
            match db::connect(&url).await {
                Ok(pool) => {
                    match account::ensure_owner(&pool, &cfg.owner_email).await {
                        Ok(account::OwnerSeed::Created) => {
                            tracing::info!("owner account created");
                        }
                        Ok(account::OwnerSeed::PasswordMissing) => {
                            tracing::warn!(
                                "AUTH_OWNER_PASSWORD is missing or out of range; owner login is not seeded"
                            );
                        }
                        Ok(account::OwnerSeed::Ready) => {}
                        Err(error) => tracing::warn!(%error, "owner account was not seeded"),
                    }
                    match crate::feed::seed_defaults(&pool).await {
                        Ok((true, _)) | Ok((_, true)) => {
                            tracing::info!("seeded empty bangumi or radar lists")
                        }
                        Ok((false, false)) => {}
                        Err(error) => tracing::warn!(%error, "default acg lists were not seeded"),
                    }
                    Some(pool)
                }
                Err(error) => {
                    tracing::error!(%error, "postgres connection failed");
                    None
                }
            }
        } else {
            tracing::warn!("DATABASE_URL is empty; persistence is disabled");
            None
        };
        Self::new(cfg, db, reqwest::Client::new())
    }

    /// 组装进程状态。限流窗口和 Go 版一致：聊天 20 次/分钟，生图 6 次/分钟，图库 60 次/分钟，账号 15 次/分钟。
    pub fn new(cfg: Config, db: Option<PgPool>, http: reqwest::Client) -> Self {
        Self {
            cfg,
            db,
            http,
            started: Instant::now(),
            chat_gate: RateGate::new(20, Duration::from_secs(60)),
            image_gate: RateGate::new(6, Duration::from_secs(60)),
            gallery_gate: RateGate::new(60, Duration::from_secs(60)),
            auth_gate: RateGate::new(15, Duration::from_secs(60)),
            sync_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn router(state: AppState) -> axum::Router {
    routes::router(state)
}
