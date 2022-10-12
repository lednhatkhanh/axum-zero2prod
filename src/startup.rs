use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::{email_client::EmailClient, routes};

pub fn get_app(pool: PgPool, email_client: EmailClient, base_url: String) -> Router {
    let email_client = Arc::new(email_client);
    Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscribe", post(routes::subscribe))
        .route("/subscriptions/confirm", get(routes::confirm))
        .layer(Extension(pool))
        .layer(Extension(email_client))
        .layer(Extension(base_url))
        .layer(TraceLayer::new_for_http())
}
