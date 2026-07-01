//! Traffic measurement types.

use serde::{Deserialize, Serialize};

/// Live traffic counters for a rule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficTotals {
    pub tcp_rx: u64,
    pub tcp_tx: u64,
    pub udp_rx: u64,
    pub udp_tx: u64,
}

impl TrafficTotals {
    pub fn total_bytes(&self) -> u64 {
        self.tcp_rx + self.tcp_tx + self.udp_rx + self.udp_tx
    }
}

/// Combined rule traffic returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSnapshot {
    pub rule_id: i64,
    pub local_port: u16,
    pub totals: TrafficTotals,
    pub quota_bytes: Option<i64>,
    pub quota_used_ratio: Option<f64>,
}
