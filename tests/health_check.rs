use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use newsletter::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    telemetry,
};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub settings: Settings,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "test";

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

// Launch the application as a background task.
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut settings = get_configuration().expect("failed to read configurtion");
    settings.database.database_name = format!(
        "{}_{}",
        settings.database.database_name,
        Uuid::new_v4().to_string()
    );
    let db_pool = configure_database(&settings.database).await;

    let server =
        newsletter::startup::run(listener, db_pool.clone()).expect("failed to bind the address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool,
        settings,
    }
}

async fn configure_database(settings: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&settings.without_db())
        .await
        .expect("failed to connect to database");
    connection
        .execute(&*format!(
            r#"CREATE DATABASE "{}";"#,
            settings.database_name
        ))
        .await
        .expect("failed to create existing test database");

    // Migrate database
    let connection_pool = PgPool::connect_with(settings.with_db())
        .await
        .expect("failed to connect to postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate the database");

    connection_pool
}

async fn stop_app(app: TestApp) {
    app.db_pool.close().await;

    // Delete database
    let settings = &app.settings.database;
    let mut connection = PgConnection::connect_with(&settings.without_db())
        .await
        .expect("failed to connect to database");
    connection
        .execute(&*format!(r#"DROP DATABASE"{}";"#, settings.database_name))
        .await
        .expect("failed to drop existing test database");
}

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    stop_app(app).await;
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT name, email FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");

    stop_app(app).await;
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    let app = spawn_app().await;
    let client = reqwest::Client::new();

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message,
        );
    }

    stop_app(app).await;
}
