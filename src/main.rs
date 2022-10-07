use std::net::SocketAddr;

use axum_zero2prod::{
    configurations::get_configuration,
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("axum_zero2prod", "info", std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
    let address = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));

    let app = get_app(connection_pool);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
