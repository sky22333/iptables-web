//! 数据库 schema 初始化（代码内维护，无需外部 SQL 文件）。

use sqlx::SqlitePool;
use tracing::info;

/// 按顺序执行的 DDL；均可重复运行（IF NOT EXISTS）。
const STATEMENTS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS rules (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        local_port      INTEGER NOT NULL UNIQUE,
        listen_host     TEXT NOT NULL DEFAULT '0.0.0.0',
        target_host     TEXT NOT NULL,
        target_port     INTEGER NOT NULL,
        enabled         INTEGER NOT NULL DEFAULT 1,
        quota_bytes     INTEGER,
        quota_period    TEXT NOT NULL DEFAULT 'none',
        period_start    TEXT,
        created_at      TEXT NOT NULL,
        updated_at      TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS traffic_counters (
        rule_id         INTEGER PRIMARY KEY REFERENCES rules(id) ON DELETE CASCADE,
        tcp_rx          INTEGER NOT NULL DEFAULT 0,
        tcp_tx          INTEGER NOT NULL DEFAULT 0,
        udp_rx          INTEGER NOT NULL DEFAULT 0,
        udp_tx          INTEGER NOT NULL DEFAULT 0,
        updated_at      TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS traffic_samples (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        rule_id         INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
        sampled_at      TEXT NOT NULL,
        tcp_rx          INTEGER NOT NULL DEFAULT 0,
        tcp_tx          INTEGER NOT NULL DEFAULT 0,
        udp_rx          INTEGER NOT NULL DEFAULT 0,
        udp_tx          INTEGER NOT NULL DEFAULT 0
    )
    "#,
    r#"
    CREATE INDEX IF NOT EXISTS idx_traffic_samples_rule_time
        ON traffic_samples(rule_id, sampled_at)
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key             TEXT PRIMARY KEY,
        value           TEXT NOT NULL
    )
    "#,
    r#"
    INSERT OR IGNORE INTO settings (key, value) VALUES ('default_start_port', '1000')
    "#,
    r#"
    INSERT OR IGNORE INTO settings (key, value) VALUES ('schema_version', '1')
    "#,
];

/// 确保数据库表结构就绪。
pub async fn init_schema(pool: &SqlitePool) -> anyhow::Result<()> {
    for sql in STATEMENTS {
        sqlx::query(sql.trim()).execute(pool).await?;
    }
    info!("数据库 schema 已就绪");
    Ok(())
}
