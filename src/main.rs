mod config;

use anyhow::Result;
use config::Config;
use tracing::info;

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

    let app = visdom_harness::build_app();

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    info!(bind_addr = %listener.local_addr()?, "visdom-harness listening");

    axum::serve(listener, app).await?;

    Ok(())
}
