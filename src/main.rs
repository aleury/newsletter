use std::io;
use std::net::TcpListener;

use sqlx::PgPool;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let configuration = get_configuration().expect("failed to read configurtion");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to postgres");

    run(listener, connection_pool)?.await
}
