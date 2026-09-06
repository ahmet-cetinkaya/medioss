//! Smoke tests over the real router: the health endpoint answers, and unknown
//! paths fall through to the static handler as 404 when nothing is built yet.

use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tempfile::TempDir;
use tower::ServiceExt;
use web_api::{config::Config, router::router, state::AppState};

/// Returns state pointing at an empty temp static dir; the guard must outlive
/// the request or the directory is removed mid-test.
fn test_state() -> (AppState, TempDir) {
    let static_dir = tempfile::tempdir().expect("create temp static dir");

    let state = AppState {
        config: Arc::new(Config {
            data_dir: PathBuf::from(static_dir.path()),
            port: 0,
            static_dir: static_dir.path().to_path_buf(),
        }),
    };

    (state, static_dir)
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (state, _static_dir) = test_state();

    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .expect("content-type header")
        .to_str()
        .expect("content-type is ascii");
    assert!(
        content_type.starts_with("application/json"),
        "expected json content-type, got `{content_type}`"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_path_returns_not_found() {
    let (state, _static_dir) = test_state();

    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
