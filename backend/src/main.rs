//! realm-web 入口：启动 HTTP 服务、数据库、转发引擎。

mod auth;
mod config;
mod domain;
mod embed;
mod engine;
mod relay;
mod routes;
mod services;
mod state;

use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::net::TcpListener;
use tracing::info;

use crate::config::AppConfig;
use crate::engine::ForwardEngine;
use crate::services::TrafficService;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "realm_web=info,tower_http=info".into()),
        )
        .init();

    let config = Arc::new(AppConfig::from_env()?);
    info!(port = config.panel_port, "正在启动 realm-web");

    let db = connect_db(&config).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let traffic = Arc::new(TrafficService::new(db.clone()));
    let engine = ForwardEngine::new();
    let state = AppState::new(config.clone(), db, traffic.clone(), engine);

    bootstrap_relays(&state).await?;
    traffic.clone().spawn_flush_task(60);
    TrafficService::spawn_quota_enforcement(state.clone(), 30);

    let listener = TcpListener::bind(("0.0.0.0", config.panel_port)).await?;
    info!(port = config.panel_port, "管理面板已就绪");

    axum::serve(listener, routes::router(state)).await?;
    Ok(())
}

async fn connect_db(config: &AppConfig) -> anyhow::Result<sqlx::SqlitePool> {
    let options: SqliteConnectOptions = config.database_url.parse()?;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options.create_if_missing(true))
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(&pool)
        .await?;
    Ok(pool)
}

/// 启动时从数据库恢复所有启用中的转发规则。
async fn bootstrap_relays(state: &AppState) -> anyhow::Result<()> {
    let rules = state.rules.list().await?;
    traffic::hydrate_meters(state, &rules).await?;

    let meters = state.traffic.all_meters().await;
    let mut engine = state.engine.lock().await;
    engine.sync_rules(&rules, &meters).await?;
    info!(count = rules.len(), "已恢复转发规则");
    Ok(())
}

mod traffic {
    use super::*;

    pub async fn hydrate_meters(state: &AppState, rules: &[domain::RuleRecord]) -> anyhow::Result<()> {
        for rule in rules {
            state.traffic.meter_for(rule).await;
        }
        state.traffic.hydrate_from_db(rules).await
    }
}
