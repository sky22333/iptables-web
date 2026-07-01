PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

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
);

CREATE TABLE IF NOT EXISTS traffic_counters (
    rule_id         INTEGER PRIMARY KEY REFERENCES rules(id) ON DELETE CASCADE,
    tcp_rx          INTEGER NOT NULL DEFAULT 0,
    tcp_tx          INTEGER NOT NULL DEFAULT 0,
    udp_rx          INTEGER NOT NULL DEFAULT 0,
    udp_tx          INTEGER NOT NULL DEFAULT 0,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS traffic_samples (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id         INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    sampled_at      TEXT NOT NULL,
    tcp_rx          INTEGER NOT NULL DEFAULT 0,
    tcp_tx          INTEGER NOT NULL DEFAULT 0,
    udp_rx          INTEGER NOT NULL DEFAULT 0,
    udp_tx          INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_traffic_samples_rule_time
    ON traffic_samples(rule_id, sampled_at);

CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('default_start_port', '1000');
