#![allow(dead_code)]
use env_logger::Env;
use rust_news_letter_server::configuration::get_configuration;
use rust_news_letter_server::startup::{run, run_0, run_1};
use rust_news_letter_server::telemetry::{get_subscriber, init_subscriber};
// use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;

// Apply to this crate,including the lib
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Use `tracing` and suits
    let subscriber = get_subscriber("rust-newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    // Use `connect_lazy` when building a docker image - only establish a connection when pool is used for the first time
    let connection = PgPool::connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to Postgres.");
    // let connection = PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // Fail faster
    // let connection_pool = PgPoolOptions::new()
    //     .connect_timeout(std::time::Duration::from_secs(2))
    //     .connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
async fn main_2() -> std::io::Result<()> {
    // `init` call `set_logger`
    // print all logs at level `info` and above if `RUST_LOG` is not set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    // `sqlx` has asynchronous interface but does not allow run multiple queries concurrently over one connection
    // so this does not compile - use `PgPool` instead

    // let connection = PgConnection::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // `PgPool` allows a handler to borrow a connection from a pool if available
    // and create a new one if not
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Use port from config file, not a random one
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}

async fn main_1() -> std::io::Result<()> {
    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    // Use port from config file, not a random one
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run_1(listener)?.await
}

async fn main_0() -> std::io::Result<()> {
    // `await` goes with `async`
    run_0("127.0.0.1:0")?.await
}
