use axum::{
    body::Body,
    http::{self, Request},
    Router,
};
use axum_zero2prod::{
    configurations::get_configuration,
    email_client::EmailClient,
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use tower::ServiceExt;
use wiremock::MockServer;

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
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: Body) {
        self.app
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
    }
}

pub async fn spawn_app(pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.email_client.base_url = email_server.uri();

        c
    };
    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
        std::time::Duration::from_millis(200),
    );

    let app = get_app(pool, email_client);

    TestApp { app, email_server }
}
