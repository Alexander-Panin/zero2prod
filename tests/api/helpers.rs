use once_cell::sync::Lazy;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
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
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client =
        EmailClient::new(configuration.email_client.base_url, sender_email);

    let server = run(listener, connection_pool.clone(), email_client)
        .expect("Failed to connect to Sqlite");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
}

async fn configure_database(config: &DatabaseSettings) -> SqlitePool {
    SqlitePoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(&config.connection_string())
        .await
        .expect("Failed to connect to Sqlite.")
}
