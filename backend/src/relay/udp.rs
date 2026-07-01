//! 基于 realm_core DNS 的 UDP 转发（端口级流量统计）。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use realm_core::dns;
use realm_core::endpoint::{Endpoint, RemoteAddr};
use realm_core::time::timeoutfut;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::warn;

use super::TrafficMeter;

struct Association {
    downstream: Arc<UdpSocket>,
    last_seen: Instant,
}

pub async fn run_udp(endpoint: Endpoint, meter: Arc<TrafficMeter>) {
    let Endpoint {
        laddr,
        raddr,
        conn_opts,
        ..
    } = endpoint;

    let socket = match UdpSocket::bind(laddr).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            warn!(%laddr, "UDP 绑定失败: {e}");
            return;
        }
    };

    let remote_addr = match resolve_first(&raddr).await {
        Ok(a) => a,
        Err(e) => {
            warn!(%raddr, "UDP 目标解析失败: {e}");
            return;
        }
    };

    let associations: Arc<Mutex<HashMap<SocketAddr, Association>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let reply_tasks: Arc<Mutex<HashMap<SocketAddr, bool>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut buf = [0u8; 65535];

    loop {
        let (n, peer) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                warn!("UDP 接收失败: {e}");
                continue;
            }
        };

        meter.add_udp_rx(n as u64);

        let downstream = {
            let mut map = associations.lock().await;
            let timeout = Duration::from_secs(conn_opts.associate_timeout.max(1) as u64);
            map.retain(|_, a| a.last_seen.elapsed() < timeout);

            if let Some(a) = map.get_mut(&peer) {
                a.last_seen = Instant::now();
                a.downstream.clone()
            } else {
                let ds = match UdpSocket::bind("0.0.0.0:0").await {
                    Ok(s) => Arc::new(s),
                    Err(e) => {
                        warn!("UDP 下游绑定失败: {e}");
                        continue;
                    }
                };
                map.insert(
                    peer,
                    Association {
                        downstream: ds.clone(),
                        last_seen: Instant::now(),
                    },
                );
                ds
            }
        };

        if downstream.send_to(&buf[..n], remote_addr).await.is_err() {
            warn!(%peer, "UDP 转发至目标失败");
        }

        let mut tasks = reply_tasks.lock().await;
        if !tasks.get(&peer).copied().unwrap_or(false) {
            tasks.insert(peer, true);
            spawn_reply(
                socket.clone(),
                downstream,
                peer,
                meter.clone(),
                reply_tasks.clone(),
                conn_opts.associate_timeout,
            );
        }
    }
}

async fn resolve_first(raddr: &RemoteAddr) -> std::io::Result<SocketAddr> {
    dns::resolve_addr(raddr)
        .await?
        .iter()
        .next()
        .ok_or_else(|| std::io::ErrorKind::NotFound.into())
}

fn spawn_reply(
    listen_sock: Arc<UdpSocket>,
    downstream: Arc<UdpSocket>,
    peer: SocketAddr,
    meter: Arc<TrafficMeter>,
    reply_tasks: Arc<Mutex<HashMap<SocketAddr, bool>>>,
    associate_timeout: usize,
) {
    tokio::spawn(async move {
        let mut buf = [0u8; 65535];
        loop {
            let recv = downstream.recv(&mut buf);
            match timeoutfut(recv, associate_timeout).await {
                Ok(Ok(n)) => {
                    if listen_sock.send_to(&buf[..n], peer).await.is_ok() {
                        meter.add_udp_tx(n as u64);
                    }
                }
                _ => break,
            }
        }
        reply_tasks.lock().await.insert(peer, false);
    });
}
