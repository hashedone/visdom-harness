use std::env;
use std::net::SocketAddr;
use std::path::Path;

use eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("BIND_ADDR `{addr}` is not a valid socket address")]
pub struct InvalidBindAddr {
    addr: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FileConfig {
    bind_addr: Option<String>,
    database_url: Option<String>,
    log_filter: Option<String>,
    anthropic: AnthropicFileConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AnthropicFileConfig {
    model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub log_filter: String,
    pub anthropic_model: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let file = Self::load_file()?;

        let bind_addr = env::var("BIND_ADDR")
            .ok()
            .or(file.bind_addr)
            .unwrap_or_else(|| "127.0.0.1:3001".to_string());

        let database_url = env::var("DATABASE_URL")
            .ok()
            .or(file.database_url)
            .unwrap_or_else(|| "sqlite://visdom.db?mode=rwc".to_string());

        let log_filter = env::var("RUST_LOG")
            .ok()
            .or(file.log_filter)
            .unwrap_or_else(|| "info,visdom_harness=debug".to_string());

        let anthropic_model = env::var("ANTHROPIC_MODEL")
            .ok()
            .or(file.anthropic.model)
            .unwrap_or_else(|| "claude-sonnet-4-6".to_string());

        bind_addr
            .parse::<SocketAddr>()
            .map_err(eyre::Report::from)
            .with_context(|| InvalidBindAddr {
                addr: bind_addr.clone(),
            })?;

        Ok(Self {
            bind_addr,
            database_url,
            log_filter,
            anthropic_model,
        })
    }

    fn load_file() -> Result<FileConfig> {
        let path = Path::new("visdom.toml");
        if !path.exists() {
            return Ok(FileConfig::default());
        }
        let text = std::fs::read_to_string(path).wrap_err("failed to read visdom.toml")?;
        toml::from_str(&text).wrap_err("failed to parse visdom.toml")
    }
}
