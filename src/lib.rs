use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use axum_macros::debug_handler;

#[debug_handler]
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

#[debug_handler]
async fn subscribe(Json(_form): Json<FormData>) -> impl IntoResponse {
    StatusCode::OK
}

pub fn get_app() -> axum::Router {
    axum::Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
}
