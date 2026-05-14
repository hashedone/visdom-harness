use std::env;
use std::net::SocketAddr;

use eyre::Result;
use eyre::WrapErr;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("BIND_ADDR `{addr}` is not a valid socket address")]
pub struct InvalidBindAddr {
    addr: String,
    #[source]
    source: std::net::AddrParseError,
}

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

        bind_addr
            .parse::<SocketAddr>()
            .map_err(|source| InvalidBindAddr {
                addr: bind_addr.clone(),
                source,
            })
            .wrap_err("invalid configuration")?;

        Ok(Self {
            bind_addr,
            database_url,
            rust_log,
        })
    }
}
