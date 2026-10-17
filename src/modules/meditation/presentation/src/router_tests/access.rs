use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn every_route_requires_an_actor() {
    let app = app();
    // A representative management route, read route, and per-user route.
    for (method, path) in [
        ("POST", "/meditation/audio"),
        ("GET", "/meditation/library"),
        ("GET", "/meditation/1"),
        ("GET", "/meditation/1/audio"),
        ("GET", "/meditation/1/download"),
        ("GET", "/meditation/1/playback-state"),
        ("GET", "/meditation/1/notes"),
        ("GET", "/meditation/programs"),
        ("GET", "/meditation/categories"),
        ("PUT", "/meditation/categories/1"),
    ] {
        let (status, _, _) = call(&app, None, method, path, vec![], Body::empty()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}

#[tokio::test]
async fn content_management_is_admin_only() {
    let app = app();
    for (method, path) in [
        ("POST", "/meditation/audio"),
        ("DELETE", "/meditation/1"),
        ("POST", "/meditation/programs"),
        ("PUT", "/meditation/programs/1"),
        ("DELETE", "/meditation/programs/1"),
        ("POST", "/meditation/programs/1/members"),
        ("DELETE", "/meditation/programs/1/members/1"),
        ("PUT", "/meditation/programs/1/category"),
        ("PUT", "/meditation/programs/1/meditations/order"),
        ("POST", "/meditation/categories"),
        ("PUT", "/meditation/categories/1"),
        ("DELETE", "/meditation/categories/1"),
    ] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            method,
            path,
            vec![("content-type", "application/json")],
            Body::from("{}"),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }
}

#[tokio::test]
async fn unknown_ids_are_404_for_every_authenticated_reader() {
    let app = app();
    for path in [
        "/meditation/999",
        "/meditation/999/audio",
        "/meditation/999/download",
        "/meditation/999/playback-state",
        "/meditation/999/notes",
    ] {
        let (status, _, _) = call(
            &app,
            Some(member_actor()),
            "GET",
            path,
            vec![],
            Body::empty(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "GET {path}");
    }
}

#[tokio::test]
async fn cross_origin_writes_are_rejected() {
    let app = app();
    let request = Request::builder()
        .method("POST")
        .uri("/meditation/categories")
        .header("origin", "http://evil.example")
        .header("content-type", "application/json")
        .extension(admin_actor())
        .body(Body::from("{\"name\":\"X\"}"))
        .unwrap();
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
