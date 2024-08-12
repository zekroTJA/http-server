mod conn;
mod readers;
mod request;
mod response;
mod statuscode;

use conn::Conn;
use tokio::net::TcpListener;
use tracing::{debug, error};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Server {
        Self { listener }
    }

    pub async fn listen(&self) -> ! {
        loop {
            match self.listener.accept().await {
                Err(err) => error!("Failed accepting connection: {}", err),
                Ok((stream, addr)) => {
                    tokio::spawn(async move {
                        debug!("Connection accepted {}", addr);
                        Conn::new(stream, addr).serve().await.unwrap();
                    });
                }
            }
        }
    }
}
