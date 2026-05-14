pub mod db;
pub mod error;
pub mod http;

use axum::Router;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn build_app(state: AppState) -> Router {
    http::router(state)
}
