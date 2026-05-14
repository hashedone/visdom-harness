pub mod error;
pub mod http;

use axum::Router;

pub fn build_app() -> Router {
    http::router()
}
