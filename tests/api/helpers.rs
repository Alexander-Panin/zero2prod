use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use zero2prod::configuration::get_configuration;
use zero2prod::startup::{build, get_connection_pool};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("test".to_string());
    init_subscriber(subscriber);
});

pub fn unix_timestamp() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros()
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let (server, port) = build(&configuration)
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{port}");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database).await,
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
}
