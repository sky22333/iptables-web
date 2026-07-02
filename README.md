## Realm 转发面板

基于 [realm](https://github.com/zhboner/realm) 的 TCP/UDP 端口转发管理面板。

## 功能

- 批量添加转发规则
- 每条规则同时转发 **TCP + UDP**
- 端口级流量统计
- JWT 登录保护
- 响应式中文界面

## 快速开始
Docker
```
docker run -d \
  --name realm-web \
  --network host \
  --restart always \
  -e AUTH_USERNAME=admin123 \
  -e AUTH_PASSWORD=admin123 \
  -e DATA_DIR=/app/data \
  -e PANEL_PORT=888 \
  -v ./data:/app/data \
  ghcr.io/sky22333/realm-web
```

或者[releases](https://github.com/sky22333/realm-web/releases)页面下载二进制文件运行

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

首次启动时，若未设置 `AUTH_USERNAME` 与 `AUTH_PASSWORD`，程序会在终端随机生成账号密码，请妥善保存。

默认面板地址：http://127.0.0.1:888


## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PANEL_PORT` | `888` | 管理面板端口 |
| `DATA_DIR` | `./data` | 数据目录（SQLite） |
| `AUTH_USERNAME` | - | 登录用户名；须与 `AUTH_PASSWORD` 同时设置 |
| `AUTH_PASSWORD` | - | 登录密码；须与 `AUTH_USERNAME` 同时设置 |
| `JWT_SECRET` | 每次启动随机 | 可选覆盖；仅内存，重启后需重新登录 |
| `DEFAULT_START_PORT` | `1000` | 自动分配起始端口 |
| `SKIP_WEB_BUILD` | - | 设为 `1` 跳过 build.rs 中的前端构建 |

## 项目结构

```
backend/          Rust 单体（API + realm_core 转发 + 嵌入前端）
frontend/         Vue 3 源码（构建后嵌入二进制）
```
