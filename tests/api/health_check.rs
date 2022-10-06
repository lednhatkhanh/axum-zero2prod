use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use axum_zero2prod::get_app;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    let response = get_app()
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
}
