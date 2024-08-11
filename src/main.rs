mod server;

use anyhow::Result;
use server::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stdout)
        .init();

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Listening on {addr} ...");
    Server::new(listener).listen().await
}
