use super::{
    request::{HeaderMap, Method, Request},
    response::ResponseBuilder,
};
use crate::server::statuscode::StatusCode;
use anyhow::Result;
use std::{
    fs::Metadata,
    io::{self, ErrorKind},
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};
use tracing::{debug, error, info};

pub struct Conn {
    stream: TcpStream,
    content_root: PathBuf,
    implicit_index: bool,
}

impl Conn {
    pub fn new(
        stream: TcpStream,
        _: SocketAddr,
        content_root: PathBuf,
        implicit_index: bool,
    ) -> Self {
        Self {
            stream,
            content_root,
            implicit_index,
        }
    }

    pub async fn serve(&mut self) -> Result<()> {
        loop {
            let r = BufReader::new(&mut self.stream);
            let Some(req) = RequestParser::new(r).parse().await? else {
                break;
            };

            info!("-> {} {}", req.method, req.path.to_string_lossy());

            if !matches!(req.method, Method::Get) {
                ResponseBuilder::new()
                    .status_code(StatusCode::MethodNotAllowed)
                    .send(&mut self.stream)
                    .await?;
                continue;
            }

            let mut path = self
                .content_root
                .join(req.path.strip_prefix("/").unwrap_or(&req.path));
            if self.implicit_index && path.is_dir() {
                path = path.join("index.html");
            }

            debug!("trying to serve file {}", path.to_string_lossy());

            match open_file(&path).await {
                Ok((f, meta)) => {
                    let mut b = ResponseBuilder::new();

                    if let Some(mime) = mime_from_path(&path) {
                        b = b.add_header("content-type", mime);
                    }

                    b.body(f, meta.len() as usize)
                        .send(&mut self.stream)
                        .await?
                }
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    ResponseBuilder::new()
                        .status_code(StatusCode::NotFound)
                        .send(&mut self.stream)
                        .await?
                }
                Err(err) => {
                    error!("opening file for response: {}", err);
                    ResponseBuilder::new()
                        .status_code(StatusCode::InternalServerError)
                        .send(&mut self.stream)
                        .await?
                }
            };
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

    pub async fn parse(&mut self) -> Result<Option<Request>> {
        let Some((proto, path, method)) = self.parse_head().await? else {
            return Ok(None);
        };

        let header = self.parse_header().await?;

        Ok(Some(Request {
            method,
            proto,
            path,
            header,
            body: None,
        }))
    }

    pub async fn parse_head(&mut self) -> Result<Option<(String, PathBuf, Method)>> {
        self.buf.clear();
        self.r.read_line(&mut self.buf).await?;

        if self.buf.trim().is_empty() {
            return Ok(None);
        }

        let mut split = self.buf.split(' ');
        let method = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no method"))?
            .into();
        let path = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no path"))?
            .into();
        let proto = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid header: no proto"))?
            .to_string();

        Ok(Some((proto, path, method)))
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

async fn open_file(path: &Path) -> io::Result<(File, Metadata)> {
    let f = File::open(path).await?;
    let meta = f.metadata().await?;
    Ok((f, meta))
}

fn mime_from_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "html" | "htm" => Some("text/html; charset=utf-8"),
        "txt" => Some("text/plain"),
        "css" => Some("text/css"),
        "js" => Some("application/javascript; charset=utf-8"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}
