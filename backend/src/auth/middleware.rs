//! Bearer 令牌鉴权中间件。

use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::validate_token;
use crate::state::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth {
        Some(v) if v.starts_with("Bearer ") => &v[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "未提供有效的认证令牌"})),
            )
                .into_response();
        }
    };

    match validate_token(token, &state.config.jwt_secret) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "令牌无效或已过期"})),
        )
            .into_response(),
    }
}
