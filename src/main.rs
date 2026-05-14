mod config;

use anyhow::Result;
use config::Config;
use tracing::info;
use visdom_harness::{AppState, db};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.rust_log)),
        )
        .json()
        .init();

    let pool = db::connect_and_migrate(&cfg.database_url).await?;
    info!(database_url = %cfg.database_url, "database ready");

    let state = AppState { pool };
    let app = visdom_harness::build_app(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    info!(bind_addr = %listener.local_addr()?, "visdom-harness listening");

    axum::serve(listener, app).await?;

    Ok(())
}
