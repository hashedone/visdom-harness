mod config;

use anyhow::Result;
use config::Config;
use tracing::info;
use visdom_harness::{db, telemetry, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_env()?;

    telemetry::init(&cfg.rust_log)?;

    let pool = db::connect_and_migrate(&cfg.database_url).await?;
    info!(database_url = %cfg.database_url, "database ready");

    let state = AppState { pool };
    let app = visdom_harness::build_app(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    info!(bind_addr = %listener.local_addr()?, "visdom-harness listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    telemetry::shutdown();

    Ok(())
}
