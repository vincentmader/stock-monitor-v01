use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::AppState;

pub async fn handler() -> StatusCode {
    StatusCode::OK
}

pub async fn ready(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ready", "db": "ok" })),
        ),
        Err(err) => {
            tracing::error!(?err, "readiness check: db unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "not_ready", "db": "error" })),
            )
        }
    }
}
