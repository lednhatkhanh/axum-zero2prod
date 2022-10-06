use axum::{
    body::Body,
    http::{self, Request},
};
use axum_zero2prod::get_app;
use fake::{faker::internet::raw::SafeEmail, locales};
use fake::{faker::name::raw::*, Fake};
use serde_json::json;
use tower::ServiceExt;

fn fake_name() -> String {
    Name(locales::EN).fake()
}

fn fake_email() -> String {
    SafeEmail(locales::EN).fake()
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = get_app();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/subscribe")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    json!({"name": fake_name(), "email": fake_email()}).to_string(),
                ))
                .expect("Failed to build request."),
        )
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    // Arrange
    let app = get_app();
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
        let response = app
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
}
