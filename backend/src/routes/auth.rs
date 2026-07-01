//! 认证相关 API。

use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::auth::issue_token;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    let ip = peer.ip();

    let remaining = state.login_limiter.check(ip);
    if remaining > 0 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(LoginResponse {
                success: false,
                token: None,
                message: Some(format!("登录尝试过多，请 {remaining} 秒后再试")),
            }),
        );
    }

    let username = body.username.trim();
    let password = body.password.trim();

    if username != state.config.auth_username || password != state.config.auth_password {
        state.login_limiter.record_failure(ip);
        tracing::warn!(%username, %ip, "登录失败");
        return (
            StatusCode::OK,
            Json(LoginResponse {
                success: false,
                token: None,
                message: Some("用户名或密码错误".into()),
            }),
        );
    }

    state.login_limiter.record_success(ip);

    match issue_token(
        username,
        &state.config.jwt_secret,
        state.config.jwt_expire_hours,
    ) {
        Ok(pair) => (
            StatusCode::OK,
            Json(LoginResponse {
                success: true,
                token: Some(pair.token),
                message: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LoginResponse {
                success: false,
                token: None,
                message: Some(format!("签发令牌失败: {e}")),
            }),
        ),
    }
}
