use axum::{routing::get, Router};
use std::sync::Arc;

use crate::AppState;

mod health;
pub mod telegram;

pub fn build(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::handler))
        .route("/ready", get(health::ready))
        .merge(telegram::router(state.clone()))
        .with_state(state)
}
