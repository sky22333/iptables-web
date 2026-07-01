//! 将面板规则映射为 realm_core Endpoint。

use std::net::SocketAddr;

use realm_core::endpoint::{BindOpts, ConnectOpts, Endpoint, RemoteAddr};

use crate::domain::RuleRecord;

/// 构建 realm 转发端点（含连接/关联选项默认值）。
pub fn endpoint_from_rule(rule: &RuleRecord) -> anyhow::Result<Endpoint> {
    let laddr: SocketAddr = format!("{}:{}", rule.listen_host, rule.local_port).parse()?;
    let raddr = RemoteAddr::DomainName(rule.target_host.clone(), rule.target_port);

    let mut conn_opts = ConnectOpts::default();
    // 与 realm 默认配置对齐的 sensible defaults
    conn_opts.connect_timeout = 10;
    conn_opts.associate_timeout = 30;
    conn_opts.tcp_keepalive = 75;
    conn_opts.tcp_keepalive_probe = 9;

    Ok(Endpoint {
        laddr,
        raddr,
        bind_opts: BindOpts::default(),
        conn_opts,
        extra_raddrs: Vec::new(),
    })
}
