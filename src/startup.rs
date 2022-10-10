use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::{email_client::EmailClient, routes};

pub fn get_app(pool: PgPool, email_client: EmailClient) -> Router {
    let email_client = Arc::new(email_client);
    Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscribe", post(routes::subscribe))
        .layer(Extension(pool))
        .layer(Extension(email_client))
        .layer(TraceLayer::new_for_http())
}
