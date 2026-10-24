use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use mood_tracker_application::ApplicationError;

use super::{ensure_browser_origin, ApiError};

const ORIGIN: &str = "https://medioss.example";

fn headers_with_origin(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_str(value).unwrap());
    headers
}

async fn status_and_body(error: ApiError) -> (StatusCode, serde_json::Value) {
    let response = error.into_response();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    )
}

#[test]
fn matching_origin_is_accepted() {
    assert!(ensure_browser_origin(&headers_with_origin(ORIGIN), ORIGIN).is_ok());
}

#[test]
fn origin_comparison_ignores_ascii_case() {
    assert!(ensure_browser_origin(&headers_with_origin("HTTPS://MEDIOSS.EXAMPLE"), ORIGIN).is_ok());
}

#[tokio::test]
async fn missing_origin_is_forbidden() {
    let error = ensure_browser_origin(&HeaderMap::new(), ORIGIN).unwrap_err();

    assert_eq!(status_and_body(error).await.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn different_origin_is_forbidden() {
    for value in ["https://evil.example", "https://medioss.example.evil", ""] {
        let error = ensure_browser_origin(&headers_with_origin(value), ORIGIN).unwrap_err();

        assert_eq!(
            status_and_body(error).await.0,
            StatusCode::FORBIDDEN,
            "{value:?}"
        );
    }
}

#[tokio::test]
async fn unauthorized_is_401() {
    assert_eq!(
        status_and_body(ApiError::unauthorized()).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn invalid_input_is_400_and_exposes_its_message() {
    let (status, body) = status_and_body(ApiError::Application(ApplicationError::InvalidInput(
        "bad score",
    )))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "input is invalid: bad score");
}

#[tokio::test]
async fn not_found_is_404_with_a_generic_body() {
    let (status, body) = status_and_body(ApiError::Application(ApplicationError::NotFound)).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "internal error");
}

#[tokio::test]
async fn repository_failure_is_500_and_hides_internals() {
    let (status, body) = status_and_body(ApiError::Application(ApplicationError::Repository(
        "database is locked".to_owned(),
    )))
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "internal error");
}
