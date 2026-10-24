use crate::test_support::{anonymous_app, authenticated_app, default_dependencies, get};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn unknown_route_is_404() {
    let response = authenticated_app(default_dependencies())
        .oneshot(get("/portability/nope"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_routes_reject_post_with_405() {
    for uri in ["/portability/export/json", "/portability/export/csv"] {
        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let response = authenticated_app(default_dependencies())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "{uri}");
    }
}

#[tokio::test]
async fn import_route_rejects_get_with_405() {
    let response = authenticated_app(default_dependencies())
        .oneshot(get("/portability/import"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn every_route_requires_an_actor() {
    for uri in ["/portability/export/json", "/portability/export/csv"] {
        let response = anonymous_app(default_dependencies())
            .oneshot(get(uri))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
    let import = Request::builder()
        .method("POST")
        .uri("/portability/import")
        .body(Body::empty())
        .unwrap();

    let response = anonymous_app(default_dependencies())
        .oneshot(import)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
