use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Executor};
use tracing::info;

use crate::error::AppError;

pub async fn connect_and_migrate(database_url: &str) -> Result<SqlitePool, AppError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!(database_url = database_url, "database migrations complete");

    Ok(pool)
}

pub async fn in_memory_pool() -> Result<SqlitePool, AppError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON").await?;
                Ok(())
            })
        })
        .connect("sqlite::memory:")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
