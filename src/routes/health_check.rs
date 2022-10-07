use axum::{http::StatusCode, response::IntoResponse};
use axum_macros::debug_handler;

#[debug_handler]
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
