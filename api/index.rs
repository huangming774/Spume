use taozhiyy_api::{router, AppState, Config};
use tower::ServiceBuilder;
use tracing_subscriber::EnvFilter;
use vercel_runtime::{axum::VercelLayer, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::from_env();
    if cfg.allow_unsigned_internal {
        tracing::warn!(
            "ACG_ALLOW_UNSIGNED_INTERNAL=true: /api/internal/* skips QStash signature checks"
        );
    }

    // Fluid compute 会复用函数实例，但不启动常驻进程专属的定时后台任务。
    let state = AppState::connect(cfg).await;
    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router(state));

    vercel_runtime::run(app).await
}
