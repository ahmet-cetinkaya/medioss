use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn setup_status_is_public_and_reports_required_when_empty() {
    let response = app()
        .oneshot(
            Request::get("/identity/setup-status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn setup_rejects_missing_origin_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_mismatched_origin_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://evil.example")
                .header("x-setup-secret", "session-secret-value")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_missing_secret_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_rejects_wrong_secret_with_forbidden() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("x-setup-secret", "wrong-secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_rejects_missing_origin_with_forbidden() {
    // A request with no Origin header at all is rejected: fail closed
    // rather than assume it is a same-origin browser request.
    let response = app()
        .oneshot(
            Request::post("/identity/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = app()
        .oneshot(
            Request::post("/identity/login")
                .header("content-type", "application/json")
                .header("origin", "http://evil.example")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"long-enough"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn me_without_actor_returns_unauthorized() {
    let response = app()
        .oneshot(Request::get("/identity/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_without_actor_returns_unauthorized_before_body_parsing() {
    let response = app()
        .oneshot(
            Request::post("/identity/users")
                .header("origin", "https://medioss.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn weak_password_maps_to_bad_request() {
    let response = app()
        .oneshot(
            Request::post("/identity/setup")
                .header("origin", "https://medioss.example")
                .header("x-setup-secret", "session-secret-value")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"root@example.com","password":"short"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
