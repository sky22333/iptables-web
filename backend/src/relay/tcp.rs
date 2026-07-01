//! Counted TCP port forwarding.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::warn;

use super::TrafficMeter;

pub async fn run_tcp(listen: std::net::SocketAddr, remote: String, meter: Arc<TrafficMeter>) {
    let listener = match TcpListener::bind(listen).await {
        Ok(l) => l,
        Err(e) => {
            warn!(%listen, "tcp bind failed: {e}");
            return;
        }
    };

    loop {
        let Ok((mut inbound, _)) = listener.accept().await else {
            continue;
        };

        let remote = remote.clone();
        let meter = meter.clone();

        tokio::spawn(async move {
            let Ok(mut outbound) = TcpStream::connect(&remote).await else {
                return;
            };

            let (mut ri, mut wi) = inbound.split();
            let (mut ro, mut wo) = outbound.split();

            let c2s = async {
                let mut buf = [0u8; 16 * 1024];
                loop {
                    let n = ri.read(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    wo.write_all(&buf[..n]).await?;
                    meter.add_tcp_rx(n as u64);
                }
                anyhow::Ok(())
            };

            let s2c = async {
                let mut buf = [0u8; 16 * 1024];
                loop {
                    let n = ro.read(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    wi.write_all(&buf[..n]).await?;
                    meter.add_tcp_tx(n as u64);
                }
                anyhow::Ok(())
            };

            let _ = tokio::try_join!(c2s, s2c);
        });
    }
}
