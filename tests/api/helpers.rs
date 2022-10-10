use axum::Router;
use axum_zero2prod::{
    configurations::get_configuration,
    email_client::EmailClient,
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use secrecy::Secret;
use sqlx::PgPool;

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test", "debug", std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test", "debug", std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub app: Router,
}

pub fn spawn_app(pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        Secret::from("asd".to_string()),
    );

    let app = get_app(pool, email_client);

    TestApp { app }
}
