use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let app = app();
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_only_routes_forbid_members() {
    let app = app();
    let payload = r#"{"name":"Whatever"}"#;
    let (headers, body) = json(payload);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "category creation is admin-only"
    );
}

#[tokio::test]
async fn mutations_require_matching_origin() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (mut headers, body) = json(payload);
    headers.retain(|(name, _)| *name != "origin");
    let mut request = Request::builder()
        .method("POST")
        .uri("/soundscape/mixes")
        .header("origin", "https://evil.example")
        .header("content-type", "application/json");
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let mut request = request.body(body).unwrap();
    request.extensions_mut().insert(member_actor());
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn listing_sounds_requires_any_signed_in_actor() {
    let app = app();
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
}
