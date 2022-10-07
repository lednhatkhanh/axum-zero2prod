use axum::Router;
use axum_zero2prod::{
    startup::get_app,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
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

    let app = get_app(pool.clone());

    TestApp { app }
}
