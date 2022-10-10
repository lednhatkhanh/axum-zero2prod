use axum::{
    body::Body,
    http::{self, Request},
};
use fake::{faker::internet::raw::SafeEmail, locales};
use fake::{faker::name::raw::*, Fake};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

fn fake_name() -> String {
    Name(locales::EN).fake()
}

fn fake_email() -> String {
    SafeEmail(locales::EN).fake()
}

#[sqlx::test]
async fn subscribe_returns_a_200_for_valid_form_data(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool.clone()).await;
    let name = fake_name();
    let email = fake_email();

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/subscribe")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    json!({"name": name, "email": email}).to_string(),
                ))
                .expect("Failed to build request."),
        )
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);

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
        let response = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/subscribe")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(invalid_body))
                    .expect("Failed to build request."),
            )
            .await
            .expect("Failed to execute request.");

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
        let response = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/subscribe")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(body))
                    .expect("Failed to build request."),
            )
            .await
            .expect("Failed to execute request.");

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
