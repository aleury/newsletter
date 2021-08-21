use std::io;
use std::net::TcpListener;

use newsletter::run;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    run(listener)?.await
}
