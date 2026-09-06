//! HTTP routing: JSON API under `/api`, everything else served from the
//! static directory (absent in dev until the frontend build lands).

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use tower_http::services::ServeDir;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let static_files = ServeDir::new(&state.config.static_dir);

    Router::new()
        .route("/api/health", get(health))
        .fallback_service(static_files)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}
