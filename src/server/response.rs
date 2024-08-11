use super::{request::HeaderMap, statuscode::StatusCode};
use anyhow::Result;
use tokio::{
    io::{AsyncRead, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Default)]
pub struct ResponseBuilder {
    status_code: StatusCode,
    header: Option<HeaderMap>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn header(mut self, header: HeaderMap) -> Self {
        self.header = Some(header);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub async fn send(self, stream: &mut TcpStream) -> Result<()> {
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

        stream.write_all("Content-Length: 0\r\n".as_bytes()).await?;

        stream.write_all("\r\n".as_bytes()).await?;
        Ok(())
    }
}
