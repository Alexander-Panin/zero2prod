use crate::configuration::DatabaseSettings;
use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> SqlitePool {
    SqlitePoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(&configuration.connection_string())
        .await
        .expect("Failed to connect to sqlite.")
}

pub async fn build(configuration: &Settings) -> Result<(Server, u16), std::io::Error> {
    let connection_pool = get_connection_pool(&configuration.database).await;
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client =
        EmailClient::new(configuration.email_client.base_url.clone(), sender_email);

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    let port = listener.local_addr().unwrap().port();
    let server = run(listener, connection_pool, email_client)?;
    Ok((server, port))
}

pub fn run(
    listener: TcpListener,
    db_pool: SqlitePool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
