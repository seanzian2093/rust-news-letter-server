use crate::routes::{greet, health_check, subscribe, subscribe_0, subscribe_1};
use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> std::io::Result<Server> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Instead of `Logger::default()`, we use `TracingLogger::default()`
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            // attach a copy of pointer to database connection to the application - i.e. only connection but multi copies across threads
            // .app_data(connection.clone())
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
// `HttpServer`does two jobs given an address - bind it and start the app
pub fn run_0(address: &str) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe_0))
    })
    .bind(address)?
    .run();

    // Return a server without running it so this is a regular function, not an `async fn`
    Ok(server)
}

// we use `std::net::TcpListener` to do the 1st job - bind the address
// so fat `run_0` and `run_1` work solely with data from the incoming request
pub fn run_1(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe_1))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

// Now uwe `app_data` to attach to application other data that are not related to the incoming request - application state
// pub fn run(listener: TcpListener, connection: PgConnection) -> std::io::Result<Server> {
pub fn run_2(listener: TcpListener, db_pool: PgPool) -> std::io::Result<Server> {
    // Wrap the connection in a smart pointer to share it between multiple handlers - thread
    // let connection = web::Data::new(connection);
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            // attach a copy of pointer to database connection to the application - i.e. only connection but multi copies across threads
            // .app_data(connection.clone())
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
