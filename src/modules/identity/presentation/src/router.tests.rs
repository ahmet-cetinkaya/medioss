use axum::http::StatusCode;
use tower::ServiceExt;

use crate::identity_router;
use crate::router_tests::helpers::{
    dependencies, NullAdmission, NullAvatarStore, NullClock, NullHasher, SeededRepository,
};

/// Empty-repository state: every route still resolves; handlers stop at the
/// origin guard or the extractors before any dependency is reached.
fn state() -> _StateAlias {
    dependencies(
        std::sync::Arc::new(SeededRepository::empty()),
        std::sync::Arc::new(NullHasher),
        std::sync::Arc::new(NullClock),
        std::sync::Arc::new(NullAdmission),
        std::sync::Arc::new(NullAvatarStore),
    )
}

type _StateAlias = crate::SharedDependencies;

fn request(method: &str, path: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method)
        .uri(path)
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn status_of(method: &str, path: &str) -> StatusCode {
    identity_router()
        .with_state(state())
        .oneshot(request(method, path))
        .await
        .unwrap()
        .status()
}

/// Every identity route answers with an auth/origin error (4xx), never 404
/// or 405, so a renamed handler shows up as an unexpected status here.
#[tokio::test]
async fn all_documented_routes_resolve() {
    let routes = [
        ("GET", "/identity/setup-status"),
        ("POST", "/identity/setup"),
        ("POST", "/identity/login"),
        ("POST", "/identity/logout"),
        ("GET", "/identity/me"),
        ("DELETE", "/identity/me"),
        ("POST", "/identity/me/password"),
        ("PATCH", "/identity/me/profile"),
        ("PUT", "/identity/me/avatar"),
        ("DELETE", "/identity/me/avatar"),
        ("POST", "/identity/users"),
        ("GET", "/identity/users"),
        ("PATCH", "/identity/users/1/status"),
        ("DELETE", "/identity/users/1"),
        ("GET", "/identity/avatar/1"),
    ];
    for (method, path) in routes {
        let status = status_of(method, path).await;
        assert!(
            status != StatusCode::NOT_FOUND && status != StatusCode::METHOD_NOT_ALLOWED,
            "{method} {path} must be routed, got {status}"
        );
    }
}

#[tokio::test]
async fn unknown_routes_are_not_found() {
    assert_eq!(status_of("GET", "/identity/nope").await, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wrong_methods_are_method_not_allowed() {
    assert_eq!(
        status_of("DELETE", "/identity/login").await,
        StatusCode::METHOD_NOT_ALLOWED
    );
}
