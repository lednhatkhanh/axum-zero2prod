use std::net::{SocketAddr, TcpListener};

use axum::http;
use axum_zero2prod::{
    configurations::get_configuration,
    email_client::EmailClient,
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use fake::{faker::internet::raw::SafeEmail, locales};
use fake::{faker::name::raw::*, Fake};
use hyper::Body;
use once_cell::sync::Lazy;
use reqwest::Url;
use sqlx::PgPool;
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
    pub client: reqwest::Client,
    pub email_server: MockServer,
    pub addr: SocketAddr,
    pub port: u16,
}

pub struct ConfirmationLinks {
    pub html: String,
    pub plain_text: String,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: hyper::Body) -> reqwest::Response {
        self.client
            .post(self.url_for("/subscribe"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(body))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn url_for(&self, path: &str) -> Url {
        let mut url = Url::parse(&format!("http://{}", self.addr)).unwrap();
        url.set_path(path);
        url
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            links[0].as_str().to_owned()
        };

        let html_link = get_link(&body["content"][0]["value"].as_str().unwrap());
        let text_link = get_link(&body["content"][1]["value"].as_str().unwrap());
        ConfirmationLinks {
            html: html_link,
            plain_text: text_link,
        }
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

    let app = get_app(pool, email_client, configuration.application.base_url);

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    TestApp {
        email_server,
        addr,
        port: addr.port(),
        client: reqwest::Client::new(),
    }
}

pub fn fake_name() -> String {
    Name(locales::EN).fake()
}

pub fn fake_email() -> String {
    SafeEmail(locales::EN).fake()
}
