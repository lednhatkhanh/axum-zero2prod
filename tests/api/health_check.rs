use axum::{body::Body, http::StatusCode};
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn health_check_works(pool: PgPool) -> sqlx::Result<()> {
    let test_app = spawn_app(pool).await;
    dbg!(test_app.url_for("/health_check"));
    let response = test_app
        .client
        .get(test_app.url_for("/health_check"))
        .body(Body::empty())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);

    let content_length_header = response.headers().get("content-length");
    assert_eq!("0", content_length_header.unwrap());

    Ok(())
}
