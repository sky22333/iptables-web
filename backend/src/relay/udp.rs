//! 带字节计数的 UDP 中继。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::warn;

use super::TrafficMeter;

const ASSOC_TIMEOUT: Duration = Duration::from_secs(30);

struct Association {
    downstream: Arc<UdpSocket>,
    last_seen: Instant,
}

pub async fn run_udp(listen: SocketAddr, remote: String, meter: Arc<TrafficMeter>) {
    let socket = Arc::new(match UdpSocket::bind(listen).await {
        Ok(s) => s,
        Err(e) => {
            warn!(%listen, "UDP 绑定失败: {e}");
            return;
        }
    });

    let remote_addr: SocketAddr = match tokio::net::lookup_host(&remote).await {
        Ok(mut iter) => match iter.next() {
            Some(a) => a,
            None => {
                warn!(%remote, "UDP 目标解析为空");
                return;
            }
        },
        Err(e) => {
            warn!(%remote, "UDP 目标解析失败: {e}");
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
            map.retain(|_, a| a.last_seen.elapsed() < ASSOC_TIMEOUT);

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

        if downstream.send_to(&buf[..n], remote_addr).await.is_ok() {
            meter.add_udp_tx(n as u64);
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
            );
        }
    }
}

fn spawn_reply(
    listen_sock: Arc<UdpSocket>,
    downstream: Arc<UdpSocket>,
    peer: SocketAddr,
    meter: Arc<TrafficMeter>,
    reply_tasks: Arc<Mutex<HashMap<SocketAddr, bool>>>,
) {
    tokio::spawn(async move {
        let mut buf = [0u8; 65535];
        loop {
            match tokio::time::timeout(ASSOC_TIMEOUT, downstream.recv(&mut buf)).await {
                Ok(Ok(n)) => {
                    meter.add_udp_rx(n as u64);
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
