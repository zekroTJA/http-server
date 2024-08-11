use super::{
    request::{HeaderMap, Method, Request},
    response::ResponseBuilder,
};
use anyhow::Result;
use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};

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

            ResponseBuilder::default().send(&mut self.stream).await?;
        }

        Ok(())
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
        let header = self.parse_header().await?;

        Ok(Request {
            method,
            proto,
            path,
            header,
            body: None,
        })
    }

    pub async fn parse_head(&mut self) -> Result<(String, PathBuf, Method)> {
        self.buf.clear();
        self.r.read_line(&mut self.buf).await?;

        let mut split = self.buf.split(' ');
        let proto = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no proto"))?
            .to_string();
        let path = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no path"))?
            .into();
        let method = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no method"))?
            .into();

        Ok((proto, path, method))
    }

    pub async fn parse_header(&mut self) -> Result<HeaderMap> {
        let mut m = HeaderMap::new();

        loop {
            self.buf.clear();
            self.r.read_line(&mut self.buf).await?;

            if self.buf.trim().is_empty() {
                break;
            }

            let Some((key, value)) = self.buf.split_once(':') else {
                anyhow::bail!("invalid header: no key-value pair");
            };

            m.insert(key, value.trim());
        }

        Ok(m)
    }
}
