//! 在本地连接侧统计字节（端口级 rx/tx）。

use std::io::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use super::TrafficMeter;

/// 包装本地 TCP 连接：read → tcp_rx，write → tcp_tx。
pub struct CountedTcpStream {
    inner: tokio::net::TcpStream,
    meter: Arc<TrafficMeter>,
}

impl CountedTcpStream {
    pub fn new(inner: tokio::net::TcpStream, meter: Arc<TrafficMeter>) -> Self {
        Self { inner, meter }
    }
}

impl AsyncRead for CountedTcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let filled_before = buf.filled().len();
        let poll = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &poll {
            let n = buf.filled().len() - filled_before;
            if n > 0 {
                self.meter.add_tcp_rx(n as u64);
            }
        }
        poll
    }
}

impl AsyncWrite for CountedTcpStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match Pin::new(&mut self.inner).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                if n > 0 {
                    self.meter.add_tcp_tx(n as u64);
                }
                Poll::Ready(Ok(n))
            }
            other => other,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// realm_io 双向转发（Linux 优先 zero-copy）。
pub async fn bidi_relay(
    local: &mut CountedTcpStream,
    remote: &mut tokio::net::TcpStream,
) -> Result<()> {
    use realm_core::realm_io;

    #[cfg(target_os = "linux")]
    {
        match realm_io::bidi_zero_copy(local, remote).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
                realm_io::bidi_copy(local, remote).await.map(|_| ())
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        realm_io::bidi_copy(local, remote).await.map(|_| ())
    }
}
