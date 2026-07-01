//! Domain types for forwarding rules and traffic accounting.

mod api;
mod rule;
mod traffic;

pub use api::{DashboardStats, UpdateRuleRequest};
pub use rule::{ForwardRule, PortAssignMode, QuotaPeriod, RuleRecord};
pub use traffic::{TrafficSnapshot, TrafficTotals};
