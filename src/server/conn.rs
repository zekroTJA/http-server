use anyhow::Result;
use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    net::TcpStream,
};

use super::request::{Method, Request};

pub struct Conn {
    stream: TcpStream,
    addr: SocketAddr,
}

impl Conn {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        Self { stream, addr }
    }

    pub async fn serve(&mut self) -> Result<()> {
        loop {
            let mut r = BufReader::new(&mut self.stream);
            let req = RequestParser::new(r).parse().await?;
        }
    }
}

struct RequestParser<R> {
    r: R,
    buf: String,
}

impl<R> RequestParser<R>
where
    R: AsyncBufReadExt,
    R: Unpin,
{
    pub fn new(r: R) -> Self {
        Self {
            r,
            buf: String::new(),
        }
    }

    pub async fn parse(&mut self) -> Result<Request> {
        let (proto, path, method) = self.parse_head().await?;

        Ok(Request {
            method,
            proto,
            path,
            header: todo!(),
            body: todo!(),
        })
    }

    pub async fn parse_head(&mut self) -> Result<(String, PathBuf, Method)> {
        self.buf.clear();
        self.r.read_line(&mut self.buf).await?;

        let mut split = self.buf.split(' ');
        let proto = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header"))?
            .to_string();
        let path = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header"))?
            .into();
        let method = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header"))?
            .into();

        Ok((proto, path, method))
    }
}
