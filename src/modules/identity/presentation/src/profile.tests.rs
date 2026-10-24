use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use super::MAX_AVATAR_BYTES;
use crate::router_tests::*;

fn request_with_actor(method: axum::http::Method, path: &str) -> axum::http::Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
    request
}

#[tokio::test]
async fn profile_patch_without_origin_is_forbidden_before_any_write() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let mut request = Request::patch("/identity/me/profile")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"display_name":"Ada"}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn profile_patch_for_an_unknown_actor_fails_unauthorized() {
    let router = seeded_app(vec![]);
    let mut request = request_with_actor(axum::http::Method::PATCH, "/identity/me/profile");
    *request.body_mut() = Body::from(r#"{"display_name":"Ada"}"#);
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(77), Role::Member));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn avatar_put_without_origin_is_forbidden_before_reading_the_body() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let mut request = Request::put("/identity/me/avatar")
        .header("content-type", "multipart/form-data; boundary=x")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn avatar_delete_without_origin_is_forbidden() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let mut request = Request::delete("/identity/me/avatar")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn avatar_limit_is_two_mebibytes() {
    assert_eq!(MAX_AVATAR_BYTES, 2 * 1024 * 1024);
}
