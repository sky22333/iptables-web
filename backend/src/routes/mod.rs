//! HTTP API 路由。

mod auth;
mod rules;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use tower_http::trace::TraceLayer;

use crate::auth::auth_middleware;
use crate::embed;
use crate::state::AppState;

/// 构建完整路由：公开 API + 受保护 API + 嵌入式前端。
pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/auth/login", post(auth::login))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/stats", get(rules::dashboard_stats))
        .route("/rules", get(rules::list_rules).post(rules::add_rules))
        .route(
            "/rules/{port}",
            patch(rules::update_rule).delete(rules::delete_rule),
        )
        .route("/rules/{port}/toggle", post(rules::toggle_rule))
        .route("/rules/{port}/traffic/reset", post(rules::reset_traffic))
        .route("/rules/batch", delete(rules::delete_batch))
        .route("/traffic", get(rules::list_traffic))
        .route("/ports/used", get(rules::used_ports))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    Router::new()
        .nest("/api", public.merge(protected))
        .route("/health", get(|| async { "ok" }))
        .fallback(embed::serve_embedded)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
