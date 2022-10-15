use axum::{extract::Query, response::IntoResponse, Extension};
use axum_macros::debug_handler;
use hyper::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[debug_handler]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    Query(parameters): Query<Parameters>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let id = match get_subscriber_id_from_token(&pool, parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match id {
        None => return StatusCode::UNAUTHORIZED,
        Some(id) => {
            if confirm_subscriber(&pool, id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }

            StatusCode::OK
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1;"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: String,
) -> Result<Option<Uuid>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1;",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(record.map(|r| r.subscriber_id))
}
