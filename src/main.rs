mod config;

use config::Config;
use eyre::Result;
use tracing::info;
use visdom_harness::AppState;
use visdom_harness::db;
use visdom_harness::llm::anthropic::AnthropicLlmClient;
use visdom_harness::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let cfg = Config::load()?;

    telemetry::init(&cfg.log_filter)?;

    let pool = db::connect_and_migrate(&cfg.database_url).await?;
    info!(database_url = %cfg.database_url, "database ready");

    let llm =
        AnthropicLlmClient::from_env(&cfg.anthropic_model).map_err(|e| eyre::eyre!("{}", e))?;

    let state = AppState { pool, llm };
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
