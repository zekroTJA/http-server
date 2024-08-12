use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};

pub trait ContentLength {
    fn len(&self) -> usize;
}

#[derive(Default)]
pub struct NoOp;

impl AsyncRead for NoOp {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Default)]
pub struct Text {
    v: String,
    pos: usize,
}

impl AsyncRead for Text {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let bytes = self.v.as_bytes();
        let remaining = bytes.len() - self.pos;

        if remaining == 0 {
            buf.set_filled(0);
            return Poll::Ready(Ok(()));
        }

        let len = buf.remaining().min(remaining);
        buf.put_slice(&bytes[self.pos..self.pos + len]);
        self.get_mut().pos += len;

        Poll::Ready(Ok(()))
    }
}

impl<S> From<S> for Text
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self {
            v: value.into(),
            ..Default::default()
        }
    }
}

impl ContentLength for NoOp {
    fn len(&self) -> usize {
        0
    }
}

impl ContentLength for Text {
    fn len(&self) -> usize {
        self.v.len()
    }
}
