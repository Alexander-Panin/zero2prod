use once_cell::sync::Lazy;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
}

pub async fn configure_database(config: &DatabaseSettings) -> SqlitePool {
    SqlitePoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(&config.connection_string())
        .await
        .expect("Failed to connect to Sqlite.")
}

fn unix_timestamp() -> u128 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros()
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("test".to_string());
    init_subscriber(subscriber);
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to connect to Sqlite");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let now = unix_timestamp();
    let name = format!("John Woo_{now}");
    let email = format!("{now}_ursula_le_guin%40ya.ru");
    let email_with_dog = format!("{now}_ursula_le_guin@ya.ru");
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("name={}&email={}", name, email))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());

    let query = sqlx::query!(
        r#"
            SELECT email, name FROM subscriptions
            where email = ? and name = ?
        "#,
        email_with_dog,
        name
    );

    let saved = query
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, email_with_dog);
    assert_eq!(saved.name, name);
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
