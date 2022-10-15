use crate::helpers::{fake_email, fake_name, spawn_app};
use axum::body::Body;
use reqwest::Url;
use serde_json::json;
use sqlx::PgPool;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_422(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool.clone()).await;

    // Act
    let response = test_app
        .client
        .get(test_app.url_for("/subscriptions/confirm"))
        .body(Body::empty())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(422, response.status().as_u16());

    Ok(())
}

#[sqlx::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(pool: PgPool) -> sqlx::Result<()> {
    // Arrange
    let test_app = spawn_app(pool.clone()).await;
    let body = Body::from(json!({"name": fake_name(), "email": fake_email()}).to_string());

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];

    let links = test_app.get_confirmation_links(&email_request);
    let raw_confirmation_link = links.html;
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    // Let's rewrite the URL to include the port
    confirmation_link.set_port(Some(test_app.port)).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    // Act
    let response = reqwest::get(confirmation_link).await.unwrap();
    // Assert
    assert_eq!(response.status().as_u16(), 200);

    Ok(())
}

#[sqlx::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber(pool: PgPool) -> sqlx::Result<()> {
    let test_app = spawn_app(pool.clone()).await;
    let name = fake_name();
    let email = fake_email();
    let body = Body::from(json!({"name": name, "email": email}).to_string());

    Mock::given(path("/mail/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];

    let raw_confirmation_link = test_app.get_confirmation_links(&email_request).html;
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(test_app.port)).unwrap();

    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "confirmed");

    Ok(())
}
