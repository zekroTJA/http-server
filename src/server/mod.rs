mod conn;
mod readers;
mod request;
mod response;
mod statuscode;

use conn::Conn;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{debug, error};

pub struct Server {
    listener: TcpListener,
    content_root: PathBuf,
    implicit_index: bool,
}

impl Server {
    pub fn new<P: Into<PathBuf>>(
        listener: TcpListener,
        content_root: P,
        implicit_index: bool,
    ) -> Server {
        Self {
            listener,
            content_root: content_root.into(),
            implicit_index,
        }
    }

    pub async fn listen(&self) -> ! {
        loop {
            match self.listener.accept().await {
                Err(err) => error!("Failed accepting connection: {}", err),
                Ok((stream, addr)) => {
                    let root = self.content_root.clone();
                    let implicit_index = self.implicit_index;
                    tokio::spawn(async move {
                        debug!("Connection accepted {}", addr);
                        Conn::new(stream, addr, root, implicit_index)
                            .serve()
                            .await
                            .unwrap();
                    });
                }
            }
        }
    }
}
