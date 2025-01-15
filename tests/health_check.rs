// Apply to `tests` crate
#![allow(dead_code)]
use once_cell::sync::Lazy;
use rust_news_letter_server::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{run, run_0, run_1},
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

#[actix_rt::test]
// `actix_rt::test` starts a new `tokio` runtime for each test function and shuts it down after the test function is done.
async fn health_check_works() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn health_check_works_1() {
    let address = spawn_app_1();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn health_check_works_0() {
    // Server
    spawn_app_0();

    // Client
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:3000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
// `spawn_app` is a regular function that returns a `Server` instance
// `*_0` use `run_0` which takes a `&str` address
fn spawn_app_0() {
    // port 0 means the OS will choose a random port at runtime
    let server = run_0("127.0.0.1:0").expect("Failed to bind address");
    // when `tokio` runtime is shut down, all tasks spawned on it are dropped automatically.
    tokio::spawn(server);
}

// This uses `run` which takes a `TcpLisenter`, spins up a server and return the address
fn spawn_app_1() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // Extract the port
    let port = listener.local_addr().unwrap().port();
    let server = run_1(listener).expect("Failed to bind address");
    // when `tokio` runtime is shut down, all tasks spawned on it are dropped automatically.
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    // Use `TEST_LOG` env var to control whether logs tests are printed to stdout
    // similar to `-- --nocapture` in `cargo test`
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked, the code in `TRACING` is executed
    // all other invocation will skip the code in `TRACING`
    Lazy::force(&TRACING);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configurate_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    // when `tokio` runtime is shut down, all tasks spawned on it are dropped automatically.
    tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}
// Allow spawn app that configurates a random data base for a test
async fn spawn_app3() -> TestApp {
    // Use `tracing` and suits
    // an issue - `init_subscriber` should be called once but is invoked in each test - causing test to fail
    let subscriber = get_subscriber(
        "rust-newsletter-test".into(),
        "debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configurate_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    // when `tokio` runtime is shut down, all tasks spawned on it are dropped automatically.
    tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}
pub async fn configurate_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

// `spawn_app` is `async` because it needs to connect to the database
async fn spawn_app_2() -> TestApp {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    // when `tokio` runtime is shut down, all tasks spawned on it are dropped automatically.
    tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[actix_rt::test]
// This test makes changes to a specific database so it needs to be run in isolation
// Otherwise, one test is dependent on the state of the database after the other test
async fn subscribe_returns_a_200_for_valid_form_data() {
    // let address = spawn_app_1();
    let address = spawn_app().await.address;
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    // connection must be mutable for us to query
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // Test query
    // `query` returns an anonymous struct with the fields specified in the query
    // generated at compile-run after verifying the query is valid
    // `query` requires to know where to find the database by DATABASE_URL env var
    // `configuration` file is for runtime, i.e., after compiled
    // for `test` and dev we can provide env var in a top level `.env` file - easier
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    //let address = spawn_app_1();
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}",
            error_message
        );
    }
}
