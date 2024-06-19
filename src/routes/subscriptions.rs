use crate::domain::{NewSubscriber, SubscriberName, SubscriberEmail};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::SqlitePool;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> { 
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?; 
        Ok(Self { email, name })
    } 
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
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
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
    let email = new_subscriber.email.as_ref();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (email, name, subscribed_at) 
            VALUES ($1, $2, $3)
        "#,
        email,
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
