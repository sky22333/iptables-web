//! 基于 realm_core DNS + realm_io 的 TCP 转发（端口级流量统计）。

use std::io::{ErrorKind, Result};
use std::sync::Arc;
use std::time::Duration;

use realm_core::dns;
use realm_core::endpoint::{ConnectOpts, Endpoint, RemoteAddr};
use tokio::net::{TcpListener, TcpStream};
use tracing::warn;

use super::counted::{bidi_relay, CountedTcpStream};
use super::TrafficMeter;

pub async fn run_tcp(endpoint: Endpoint, meter: Arc<TrafficMeter>) {
    let Endpoint {
        laddr,
        raddr,
        conn_opts,
        ..
    } = endpoint;

    let listener = match TcpListener::bind(laddr).await {
        Ok(l) => l,
        Err(e) => {
            warn!(%laddr, "TCP 绑定失败: {e}");
            return;
        }
    };

    loop {
        let Ok((inbound, peer)) = listener.accept().await else {
            continue;
        };
        let _ = inbound.set_nodelay(true);

        let raddr = raddr.clone();
        let conn_opts = conn_opts.clone();
        let meter = meter.clone();

        tokio::spawn(async move {
            let remote = match connect_tcp(&raddr, &conn_opts).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(%peer, %raddr, "TCP 连接目标失败: {e}");
                    return;
                }
            };

            let mut local = CountedTcpStream::new(inbound, meter);
            let mut remote = remote;
            if let Err(e) = bidi_relay(&mut local, &mut remote).await {
                tracing::debug!(%peer, "TCP 转发结束: {e}");
            }
        });
    }
}

async fn connect_tcp(raddr: &RemoteAddr, opts: &ConnectOpts) -> Result<TcpStream> {
    let timeout_secs = opts.connect_timeout;
    let mut last_err = None;

    for addr in dns::resolve_addr(raddr).await?.iter() {
        let connect_fut = TcpStream::connect(addr);
        let result = if timeout_secs == 0 {
            connect_fut.await
        } else {
            match tokio::time::timeout(Duration::from_secs(timeout_secs as u64), connect_fut).await {
                Ok(r) => r,
                Err(_) => Err(ErrorKind::TimedOut.into()),
            }
        };

        match result {
            Ok(stream) => {
                let _ = stream.set_nodelay(true);
                return Ok(stream);
            }
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| ErrorKind::NotConnected.into()))
}
