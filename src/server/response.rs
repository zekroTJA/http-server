use super::{
    readers::{ContentLength, NoOp},
    request::HeaderMap,
    statuscode::StatusCode,
};
use anyhow::Result;
use tokio::{
    io::{self, AsyncRead, AsyncWriteExt},
    net::TcpStream,
};
use tracing::debug;

#[derive(Default)]
struct Body<B> {
    body: B,
    size: usize,
}

#[derive(Default)]
pub struct ResponseBuilder<B = NoOp> {
    status_code: StatusCode,
    header: Option<HeaderMap>,
    body: Body<B>,
}

impl<B> ResponseBuilder<B> {
    pub fn status_code(self, status_code: StatusCode) -> Self {
        Self {
            status_code,
            ..self
        }
    }

    pub fn header(self, header: HeaderMap) -> Self {
        Self {
            header: Some(header),
            ..self
        }
    }

    pub fn body(self, body: B, size: usize) -> ResponseBuilder<B>
    where
        B: AsyncRead,
    {
        ResponseBuilder {
            body: Body { body, size },
            header: self.header,
            status_code: self.status_code,
        }
    }

    pub fn body_with_len(self, body: B) -> ResponseBuilder<B>
    where
        B: AsyncRead,
        B: ContentLength,
    {
        let ln = body.len();
        self.body(body, ln)
    }

    pub async fn send(mut self, stream: &mut TcpStream) -> Result<()>
    where
        B: AsyncRead + Unpin,
    {
        stream
            .write_all(
                format!(
                    "HTTP/1.1 {} {}\r\n",
                    self.status_code.code(),
                    self.status_code
                )
                .as_bytes(),
            )
            .await?;

        if let Some(header) = self.header {
            for (k, v) in header.into_iter() {
                stream.write_all(format!("{k}: {v}\r\n").as_bytes()).await?;
            }
        }

        stream
            .write_all(format!("Content-Length: {}\r\n", self.body.size).as_bytes())
            .await?;

        stream.write_all(b"\r\n").await?;

        io::copy(&mut self.body.body, stream).await?;

        debug!("Response served!");

        Ok(())
    }
}
