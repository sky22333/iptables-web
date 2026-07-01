//! 从环境变量加载应用配置。

use std::net::IpAddr;
use std::path::{Path, PathBuf};

/// 管理面板与中继引擎的运行时配置。
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub panel_port: u16,
    pub data_dir: PathBuf,
    pub database_url: String,
    pub auth_username: String,
    pub auth_password: String,
    pub jwt_secret: String,
    pub jwt_expire_hours: i64,
    pub default_start_port: u16,
    pub listen_host: IpAddr,
    pub reserved_ports: Vec<u16>,
}

impl AppConfig {
    /// 读取环境变量；本地开发默认 `./data`，容器内请设置 `DATA_DIR=/app/data`。
    pub fn from_env() -> anyhow::Result<Self> {
        let data_dir = std::env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("realm-web.db");
        let database_url = sqlite_url(&db_path);

        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("未设置 JWT_SECRET，重启后登录令牌将失效");
            uuid::Uuid::new_v4().to_string()
        });

        let listen_host: IpAddr = std::env::var("LISTEN_HOST")
            .unwrap_or_else(|_| "0.0.0.0".into())
            .parse()?;

        let reserved_ports = parse_port_list(
            &std::env::var("RESERVED_PORTS").unwrap_or_else(|_| "22,80,443,888".into()),
        );

        Ok(Self {
            panel_port: env_u16("PANEL_PORT", 888),
            data_dir,
            database_url,
            auth_username: std::env::var("AUTH_USERNAME").unwrap_or_else(|_| "admin".into()),
            auth_password: std::env::var("AUTH_PASSWORD").unwrap_or_else(|_| "password".into()),
            jwt_expire_hours: env_i64("JWT_EXPIRE_HOURS", 5),
            default_start_port: env_u16("DEFAULT_START_PORT", 1000),
            listen_host,
            reserved_ports,
            jwt_secret,
        })
    }
}

fn sqlite_url(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    format!("sqlite:///{normalized}?mode=rwc")
}

fn env_u16(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_port_list(raw: &str) -> Vec<u16> {
    raw.split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}
