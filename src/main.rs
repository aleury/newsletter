use std::io;
use std::net::TcpListener;

use sqlx::PgPool;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use newsletter::telemetry;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let subscriber = telemetry::get_subscriber("newsletter", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configurtion");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("failed to connect to postgres");

    run(listener, connection_pool)?.await
}
