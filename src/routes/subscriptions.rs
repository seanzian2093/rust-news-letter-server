use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing_futures::Instrument;
use uuid::Uuid;

// Define a struct that represents the data that a user submits
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// `#[tracing_instrument]` creates a span at the beginning of the function invocation
// automatically attach all arguments passed to the function to the span, e.g. `form`
// skip the `form` and `pool` arguments, not displaying
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
    subscriber_email = %form.email,
    subscriber_name= %form.name
    )
    )]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

// Use `web::Data` to extract value from application state
// by looking at its type, i.e. - `PgConnection`
// if there is one of such type, retrive and pass it to the handler
pub async fn subscribe_3(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        // `%` means use its `Display` impl
        request_id = %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    );
    let _request_span_guard = request_span.enter();
    // Use `span` in `tracing` to represent a whole HTTP request
    // Use `tracing-feature` to enable no mixxing of spans from multiple requests/futures
    //
    let query_span = tracing::info_span!("Saving new subscriber details in database");

    let res = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        request_id,
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    // Attach a query span to the future of the query
    .instrument(query_span)
    .await;

    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            // out of `query_span`
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn subscribe_2(
    form: web::Form<FormData>,
    // connection: web::Data<PgConnection>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // log- log infomation is created in `subscribe` handler
    // so `actix_web::middleware::Logger` in `startup.rs` is not aware of them
    // ie. it is not aware of `request_id` we created here
    // we could modify/build on `actix_web::middleware::Logger` to include `request_id` in logs
    // but no scalable
    let request_id = Uuid::new_v4();
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    log::info!(
        "request_id {} - Saving new subscriber details in database",
        request_id
    );
    // Write to database
    let res = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        request_id,
        form.email,
        form.name,
        Utc::now()
    )
    // .execute(connection.get_ref())
    .execute(pool.get_ref())
    .await;

    match res {
        // Return a 200 OK response
        Ok(_) => {
            log::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // use {:?} gives raw view of error - or {:#?} for pretty print
            log::error!(
                "request_id {} - Failed to execute query: {:?}",
                e,
                request_id
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Update the `subscribe` function to accept the form data
// when input is not deserilizable, it returns a 400 Bad Request as out test expects
pub async fn subscribe_1(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn subscribe_0() -> HttpResponse {
    HttpResponse::Ok().finish()
}
