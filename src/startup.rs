use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::routes;

pub fn get_app(pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscribe", post(routes::subscribe))
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
}
