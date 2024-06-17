use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<SqlitePool>) -> impl Responder {
    match insert_subscriber(&form, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(form: &FormData, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (email, name, subscribed_at) 
            VALUES ($1, $2, $3)
        "#,
        form.email,
        form.name,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
