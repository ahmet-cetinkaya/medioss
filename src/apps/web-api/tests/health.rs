//! Smoke tests over the real router: the health endpoint answers, unknown API
//! paths answer in JSON, and other unknown paths fall through to the static
//! handler as 404 when nothing is built yet.

use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
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

async fn get(uri: &str) -> Response<Body> {
    let (state, _static_dir) = test_state();

    router(state)
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("router response")
}

fn assert_json_content_type(response: &Response<Body>) {
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
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let response = get("/api/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_json_content_type(&response);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_returns_json_not_found() {
    let response = get("/api/does-not-exist").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_json_content_type(&response);
}

#[tokio::test]
async fn unknown_path_returns_not_found() {
    let response = get("/nonexistent").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
