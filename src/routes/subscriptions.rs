use std::sync::Arc;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use axum_macros::debug_handler;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(pool, form, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    Extension(pool): Extension<PgPool>,
    Extension(email_client): Extension<Arc<EmailClient>>,
    Json(form): Json<FormData>,
) -> impl IntoResponse {
    let new_subscriber: NewSubscriber = match form.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return StatusCode::UNPROCESSABLE_ENTITY,
    };

    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let res = email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            "Welcome to our newsletter",
            "Welcome to our newsletter",
        )
        .await;

    if res.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, 'confirmed')",
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
