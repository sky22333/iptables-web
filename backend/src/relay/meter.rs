//! In-memory traffic counters with atomic updates.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::TrafficTotals;

#[derive(Debug, Default)]
pub struct TrafficMeter {
    pub tcp_rx: AtomicU64,
    pub tcp_tx: AtomicU64,
    pub udp_rx: AtomicU64,
    pub udp_tx: AtomicU64,
}

impl TrafficMeter {
    pub fn add_tcp_rx(&self, n: u64) {
        self.tcp_rx.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_tcp_tx(&self, n: u64) {
        self.tcp_tx.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_udp_rx(&self, n: u64) {
        self.udp_rx.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_udp_tx(&self, n: u64) {
        self.udp_tx.fetch_add(n, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> TrafficTotals {
        TrafficTotals {
            tcp_rx: self.tcp_rx.load(Ordering::Relaxed),
            tcp_tx: self.tcp_tx.load(Ordering::Relaxed),
            udp_rx: self.udp_rx.load(Ordering::Relaxed),
            udp_tx: self.udp_tx.load(Ordering::Relaxed),
        }
    }
}
