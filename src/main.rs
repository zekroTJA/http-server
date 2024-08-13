mod config;
mod server;

use anyhow::Result;
use config::Config;
use server::Server;
use std::env::{self, current_dir};
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

#[cfg(test)]
mod test {
    use crate::server::Server;
    use std::time::Duration;
    use tokio::{fs::File, io::AsyncReadExt, time::sleep};

    #[tokio::test]
    async fn integration_test() {
        let addr = "127.0.0.1:18735";

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            Server::new(listener, "content", true).listen().await
        });
        sleep(Duration::from_millis(100)).await;

        let res = reqwest::get(format!("http://{addr}")).await.unwrap();
        assert!(res.status().is_success());
        assert_eq!(
            res.headers()
                .get("Content-Type")
                .map(|v| v.to_str().unwrap()),
            Some("text/html; charset=utf-8")
        );

        let mut index_contents = String::new();
        File::open("content/index.html")
            .await
            .unwrap()
            .read_to_string(&mut index_contents)
            .await
            .unwrap();

        assert_eq!(res.text().await.unwrap(), index_contents);

        let res = reqwest::get(format!("http://{addr}/does-not-exist"))
            .await
            .unwrap();
        assert_eq!(res.status().as_u16(), 404);
        assert_eq!(res.text().await.unwrap(), "Not Found");
    }
}
