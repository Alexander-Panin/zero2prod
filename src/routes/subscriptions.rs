use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::{Sqlite, Transaction};
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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into()?;
    let mut transaction = pool.begin().await?;
    insert_subscriber(&new_subscriber, &mut transaction).await?;
    // if insert_subscriber(&new_subscriber, &mut transaction).await.is_err() {
    //     return HttpResponse::InternalServerError().finish();
    // }
    transaction.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let name = new_subscriber.name.as_ref();
    let email = new_subscriber.email.as_ref();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (email, name, subscribed_at, status) 
            VALUES ($1, $2, $3, 'pending_confirmation')
        "#,
        email,
        name,
        now
    )
    .execute(transaction.as_mut())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[derive(Debug)]
pub enum SubscribeError {
    ValidationError(String),
    DatabaseError(sqlx::Error),
}

impl std::error::Error for SubscribeError {}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for SubscribeError {
    fn from(e: sqlx::Error) -> Self {
        Self::DatabaseError(e)
    }
}

impl From<String> for SubscribeError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create a new subscriber.")
    }
}
