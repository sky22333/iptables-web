//! 规则更新请求。

use serde::Deserialize;

use crate::domain::QuotaPeriod;

/// 更新单条转发规则（仅更新提供的字段）。
#[derive(Debug, Default, Deserialize)]
pub struct UpdateRuleRequest {
    pub target_host: Option<String>,
    pub target_port: Option<u16>,
    pub enabled: Option<bool>,
    /// 流量配额（GB）。
    pub quota_gb: Option<f64>,
    /// 设为 true 时清除配额限制。
    #[serde(default)]
    pub unset_quota: Option<bool>,
    pub quota_period: Option<String>,
}

impl UpdateRuleRequest {
    pub fn resolve_quota_bytes(&self, current: Option<i64>) -> Option<i64> {
        if self.unset_quota == Some(true) {
            return None;
        }
        if let Some(gb) = self.quota_gb {
            return Some((gb * 1024.0 * 1024.0 * 1024.0) as i64);
        }
        current
    }

    pub fn parse_quota_period(&self) -> Option<QuotaPeriod> {
        self.quota_period.as_ref().map(|s| QuotaPeriod::parse(s))
    }
}

/// 面板统计摘要。
#[derive(Debug, serde::Serialize)]
pub struct DashboardStats {
    pub rule_count: usize,
    pub active_count: usize,
    pub total_traffic_bytes: u64,
    pub quota_blocked_count: usize,
}
