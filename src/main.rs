mod config;

use anyhow::Result;
use config::Config;
use tracing::info;

fn main() -> Result<()> {
    let cfg = Config::from_env()?;

    // Initialize tracing before any other logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.rust_log)),
        )
        .json()
        .init();

    info!(bind_addr = %cfg.bind_addr, database_url = %cfg.database_url, "visdom-harness config loaded");

    // Real wiring (Axum, sqlx pool, OTel init) lands in T02–T04
    info!("startup complete (skeleton — HTTP server wired in T02)");

    Ok(())
}
