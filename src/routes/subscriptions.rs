use crate::domain::{NewSubscriber, SubscriberName};
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
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name,
    };

    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let name = new_subscriber.name.as_ref();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (email, name, subscribed_at) 
            VALUES ($1, $2, $3)
        "#,
        new_subscriber.email,
        name,
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
