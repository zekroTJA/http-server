mod config;
mod server;

use std::env::{self, current_dir};

use anyhow::Result;
use config::Config;
use server::Server;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let cfg_path = args.nth(1).unwrap_or_else(|| "config.toml".into());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stdout)
        .init();

    let cfg = Config::parse(&cfg_path)?;
    debug!("config: {cfg:?}");

    let addr = &cfg.server.address.unwrap_or_else(|| "0.0.0.0:80".into());

    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Listening on {addr} ...");

    let content_dir = match cfg.server.content_root {
        Some(v) => v,
        None => current_dir()?,
    };

    Server::new(listener, content_dir, cfg.server.implicit_index)
        .listen()
        .await
}
