use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

/// A readable blob handed back by the store.
///
/// Opaque on purpose: the store is filesystem-backed today, so returning
/// `tokio::fs::File` would pin the public contract to that backing and force
/// every future infrastructure test onto real disk. A newtype rather than
/// `impl AsyncRead` because callers must be able to name the type to hold it in
/// their own port structs (docs/ARCHITECTURE.md, `AudioStream`).
pub struct MediaStream(Pin<Box<dyn AsyncRead + Send>>);

impl MediaStream {
    pub fn new(reader: impl AsyncRead + Send + 'static) -> Self {
        Self(Box::pin(reader))
    }

    /// The boxed reader, for callers (tests, adapters) that need to drive it
    /// directly instead of through the AsyncRead impl.
    pub fn reader(self) -> Pin<Box<dyn AsyncRead + Send>> {
        self.0
    }
}

impl AsyncRead for MediaStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.0.as_mut().poll_read(cx, buf)
    }
}
