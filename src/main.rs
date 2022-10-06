use axum_zero2prod::get_app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = get_app();
    let address = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
