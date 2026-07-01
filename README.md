# Realm 转发面板

基于 Rust 的 TCP/UDP 端口转发管理面板。前端构建产物嵌入单一二进制，无需单独部署静态资源。

## 功能

- 批量添加转发规则（自动 / 指定起始 / 手动端口）
- 每条规则同时转发 **TCP + UDP**
- 实时流量统计与 SQLite 持久化
- JWT 登录保护
- 响应式中文界面（shadcn-vue 风格）

## 快速开始

### 环境要求

- Rust 1.85+（edition 2024）
- Node.js 20+

### 本地开发

```bash
# 安装前端依赖并构建（cargo build 会自动触发，也可手动）
cd frontend && npm install && npm run build && cd ..

# 编译并运行
cd backend
cargo run
```

默认账号：`admin` / `password`（通过环境变量 `AUTH_USERNAME`、`AUTH_PASSWORD` 修改）

面板地址：http://127.0.0.1:888

### Docker

```bash
docker compose up -d --build
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PANEL_PORT` | `888` | 管理面板端口 |
| `DATA_DIR` | `./data` | 数据目录（SQLite） |
| `AUTH_USERNAME` | `admin` | 登录用户名 |
| `AUTH_PASSWORD` | `password` | 登录密码 |
| `JWT_SECRET` | 随机 | JWT 签名密钥 |
| `DEFAULT_START_PORT` | `1000` | 自动分配起始端口 |
| `RESERVED_PORTS` | `22,80,443,888` | 保留端口 |
| `SKIP_WEB_BUILD` | - | 设为 `1` 跳过 build.rs 中的前端构建 |

## 项目结构

```
backend/          Rust 单体服务（API + 中继 + 嵌入前端）
frontend/         Vue 3 源码（构建后嵌入二进制）
```

## 许可证

MIT
