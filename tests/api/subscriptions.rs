use axum::body::Body;

use serde_json::json;
use sqlx::PgPool;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{fake_email, fake_name, spawn_app, ConfirmationLinks};

#[sqlx::test]
async fn subscribe_returns_a_200_for_valid_form_data(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool.clone()).await;
    let name = fake_name();
    let email = fake_email();
    let body = json!({"name": name, "email": email}).to_string();

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    let response = test_app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    Ok(())
}

#[sqlx::test]
async fn subscribe_persists_the_new_subscriber(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool.clone()).await;
    let email = fake_email();
    let name = fake_name();
    let body = json!({"name": name, "email": email}).to_string();

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "pending_confirmation");

    Ok(())
}

#[sqlx::test]
async fn subscribe_returns_a_422_when_data_is_missing(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool).await;
    let test_cases = vec![
        (
            json!({ "name": fake_name() }).to_string(),
            "missing the email",
        ),
        (
            json!({ "email": fake_email() }).to_string(),
            "missing the name",
        ),
        (json!({}).to_string(), "missing both name and email"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }

    Ok(())
}

#[sqlx::test]
async fn subscribe_returns_a_422_when_fields_are_present_but_invalid(
    pool: PgPool,
) -> sqlx::Result<()> {
    let test_app = spawn_app(pool).await;
    let test_cases = vec![
        (
            json!({"name": "","email": fake_email()}).to_string(),
            "empty name",
        ),
        (
            json!({"name": fake_name(),"email": ""}).to_string(),
            "empty email",
        ),
        (
            json!( {"name": fake_name(),"email": fake_name()}).to_string(),
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not return a 422 UNPROCESSABLE ENTITY when the payload was {}.",
            description
        );
    }

    Ok(())
}

#[sqlx::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data(pool: PgPool) -> sqlx::Result<()> {
    let test_app = spawn_app(pool).await;
    let body = Body::from(json!({"name": fake_name(), "email": fake_email()}).to_string());

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    // Act
    test_app.post_subscriptions(body.into()).await;

    Ok(())
}

#[sqlx::test]
async fn subscribe_sends_a_confirmation_email_with_a_link(pool: PgPool) -> sqlx::Result<()> {
    let test_app = spawn_app(pool).await;
    let body = Body::from(json!({"name": fake_name(), "email": fake_email()}).to_string());

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;

    // Assert
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];

    let ConfirmationLinks { html, plain_text } = test_app.get_confirmation_links(&email_request);

    assert_eq!(html, plain_text);

    Ok(())
}
