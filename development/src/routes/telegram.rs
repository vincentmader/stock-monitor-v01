use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use std::sync::Arc;

use crate::AppState;

/// Mounts the Telegram webhook endpoint.
/// Full handler is implemented in M8; this skeleton accepts and drops updates.
pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhook/telegram", post(webhook))
        .with_state(state)
}

async fn webhook(
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<serde_json::Value>,
) -> StatusCode {
    StatusCode::OK
}
