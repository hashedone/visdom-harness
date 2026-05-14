use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://visdom.db?mode=rwc".to_string());
        let rust_log =
            env::var("RUST_LOG").unwrap_or_else(|_| "info,visdom_harness=debug".to_string());

        // Validate bind_addr parses as a socket address
        bind_addr
            .parse::<std::net::SocketAddr>()
            .with_context(|| format!("BIND_ADDR is not a valid socket address: {bind_addr}"))?;

        Ok(Self {
            bind_addr,
            database_url,
            rust_log,
        })
    }
}
