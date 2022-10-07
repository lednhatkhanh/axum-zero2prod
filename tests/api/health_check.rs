use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn health_check_works(pool: PgPool) -> sqlx::Result<()> {
    let test_app = spawn_app(pool);
    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/health_check")
                .body(Body::empty())
                .expect("Failed to build request."),
        )
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);

    let content_length_header = response.headers().get("content-length");
    assert_eq!("0", content_length_header.unwrap());

    Ok(())
}
