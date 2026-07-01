//! 共享应用状态。

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::auth::LoginRateLimiter;
use crate::config::AppConfig;
use crate::engine::ForwardEngine;
use crate::services::{RuleService, TrafficService};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub rules: RuleService,
    pub traffic: Arc<TrafficService>,
    pub engine: Arc<Mutex<ForwardEngine>>,
    pub login_limiter: Arc<LoginRateLimiter>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        db: sqlx::SqlitePool,
        traffic: Arc<TrafficService>,
        engine: ForwardEngine,
    ) -> Self {
        let rules = RuleService::new(db, config.clone());
        Self {
            config,
            rules,
            traffic,
            engine: Arc::new(Mutex::new(engine)),
            login_limiter: Arc::new(LoginRateLimiter::new()),
        }
    }
}
