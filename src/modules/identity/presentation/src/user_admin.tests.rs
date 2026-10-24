use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use crate::router_tests::*;

fn actor_request(method: &str, path: &str, role: Role, body: &str) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), role));
    request
}

#[tokio::test]
async fn users_list_is_admin_only() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);

    let member_response = router
        .clone()
        .oneshot(actor_request("GET", "/identity/users", Role::Member, ""))
        .await
        .unwrap();
    assert_eq!(member_response.status(), StatusCode::FORBIDDEN);

    let admin_response = router
        .oneshot(actor_request("GET", "/identity/users", Role::Admin, ""))
        .await
        .unwrap();
    assert_eq!(admin_response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(admin_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed[0]["email"], "member@example.com");
    assert_eq!(listed[0]["role"], "member");
    assert!(listed[0]["disabled"].is_boolean());
    assert!(listed[0]["must_change_password"].is_boolean());
}

#[tokio::test]
async fn creating_a_user_is_member_forbidden() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let payload = r#"{"email":"new@medioss.test","password":"first-login-pass"}"#;

    let member_response = router
        .oneshot(actor_request(
            "POST",
            "/identity/users",
            Role::Member,
            payload,
        ))
        .await
        .unwrap();

    assert_eq!(member_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn creating_a_user_with_a_weak_password_fails_before_the_repository() {
    // The in-memory harness refuses inserts, so exercise the domain rule:
    // parsing rejects a short password before any insert is attempted, and
    // the failure surfaces as 400, not 500.
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let payload = r#"{"email":"new@medioss.test","password":"short"}"#;

    let response = router
        .oneshot(actor_request(
            "POST",
            "/identity/users",
            Role::Admin,
            payload,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
