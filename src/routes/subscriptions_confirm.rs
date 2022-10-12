use axum::{extract::Query, response::IntoResponse};
use axum_macros::debug_handler;
use hyper::StatusCode;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[debug_handler]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(Query(_parameters): Query<Parameters>) -> impl IntoResponse {
    StatusCode::OK
}
