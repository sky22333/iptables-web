//! High-performance counted TCP/UDP relay (traffic metering at copy layer).

mod meter;
mod tcp;
mod udp;

pub use meter::TrafficMeter;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::domain::RuleRecord;

/// Handles for a running counted relay on one local port.
pub struct RelayHandle {
    pub tcp: Option<JoinHandle<()>>,
    pub udp: Option<JoinHandle<()>>,
}

impl RelayHandle {
    pub fn abort(self) {
        if let Some(h) = self.tcp {
            h.abort();
        }
        if let Some(h) = self.udp {
            h.abort();
        }
    }
}

/// Start TCP and UDP relays for a rule with byte counting.
pub fn start_rule(
    rule: &RuleRecord,
    meter: Arc<TrafficMeter>,
) -> anyhow::Result<RelayHandle> {
    let listen: SocketAddr = format!("{}:{}", rule.listen_host, rule.local_port).parse()?;
    let remote = format!("{}:{}", rule.target_host, rule.target_port);

    let tcp = tokio::spawn(tcp::run_tcp(listen, remote.clone(), meter.clone()));
    let udp = tokio::spawn(udp::run_udp(listen, remote, meter));

    Ok(RelayHandle {
        tcp: Some(tcp),
        udp: Some(udp),
    })
}
