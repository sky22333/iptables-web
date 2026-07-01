//! 共享应用状态。

use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::engine::ForwardEngine;
use crate::services::{RuleService, TrafficService};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: SqlitePool,
    pub rules: RuleService,
    pub traffic: Arc<TrafficService>,
    pub engine: Arc<Mutex<ForwardEngine>>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        db: SqlitePool,
        traffic: Arc<TrafficService>,
        engine: ForwardEngine,
    ) -> Self {
        let rules = RuleService::new(db.clone(), config.clone());
        Self {
            config,
            db,
            rules,
            traffic,
            engine: Arc::new(Mutex::new(engine)),
        }
    }
}
