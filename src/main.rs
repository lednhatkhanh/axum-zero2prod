use std::net::SocketAddr;

use axum_zero2prod::{
    configurations::get_configuration,
    email_client::EmailClient,
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("axum_zero2prod", "info", std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let address: SocketAddr = address.parse().expect("Failed to parse address.");

    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
        timeout,
    );

    let app = get_app(connection_pool, email_client);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
