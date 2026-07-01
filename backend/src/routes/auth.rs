//! 认证相关 API。

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
    Json(body): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    if body.username != state.config.auth_username || body.password != state.config.auth_password {
        return (
            StatusCode::UNAUTHORIZED,
            Json(LoginResponse {
                success: false,
                token: None,
                message: Some("用户名或密码错误".into()),
            }),
        );
    }

    match issue_token(
        &body.username,
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
