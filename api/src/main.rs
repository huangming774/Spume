use taozhiyy_api::{router, AppState, Config};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("taozhiyy-api: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let cfg = Config::from_env();
    let addr = taozhiyy_api::config::bind_addr(&cfg.bind);
    let state = AppState::connect(cfg).await;
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|error| format!("bind {addr}: {error}"))?;
    tracing::info!(%addr, "taozhiyy-api listening");
    axum::serve(listener, app)
        .await
        .map_err(|error| format!("serve: {error}"))?;
    Ok(())
}
