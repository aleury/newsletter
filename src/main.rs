use std::io;
use std::net::TcpListener;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use newsletter::telemetry;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let subscriber = telemetry::get_subscriber("newsletter", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configurtion");
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(config.database.with_db())
        .await
        .expect("failed to connect to postgres");

    run(listener, connection_pool)?.await
}
