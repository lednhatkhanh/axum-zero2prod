use axum::{http::StatusCode, response::IntoResponse, routing::get};
use axum_macros::debug_handler;

#[debug_handler]
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub fn get_app() -> axum::Router {
    axum::Router::new().route("/health_check", get(health_check))
}
