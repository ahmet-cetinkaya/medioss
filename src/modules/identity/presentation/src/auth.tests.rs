use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use crate::router_tests::*;
use crate::Principal;

fn json_request(method: &str, path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

#[tokio::test]
async fn setup_status_is_public_and_answers_the_flag() {
    let response = app()
        .oneshot(json_request("GET", "/identity/setup-status", ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status["setup_required"], true);

    let seeded = seeded_app(vec![user(1, "root@example.com", Role::Admin)]);
    let response = seeded
        .oneshot(json_request("GET", "/identity/setup-status", ""))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status["setup_required"], false);
}

#[tokio::test]
async fn setup_rejects_wrong_setup_secret_even_with_a_matching_origin() {
    let mut request = json_request(
        "POST",
        "/identity/setup",
        r#"{"email":"root@example.com","password":"long-enough"}"#,
    );
    request
        .headers_mut()
        .insert("x-setup-secret", "wrong-secret".parse().unwrap());

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn setup_without_origin_is_forbidden() {
    let request = Request::post("/identity/setup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"root@example.com","password":"long-enough"}"#,
        ))
        .unwrap();

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn me_reports_the_principal_snapshot_without_a_repository_hit() {
    let mut request = json_request("GET", "/identity/me", "");
    request.extensions_mut().insert(Principal {
        id: 9,
        email: "snapshot@medioss.test".to_owned(),
        role: Role::Member,
        must_change_password: true,
        auth_version: 7,
        display_name: Some("Ada".to_owned()),
        has_avatar: false,
    });

    let response = app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let me: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(me["id"], 9);
    assert_eq!(me["email"], "snapshot@medioss.test");
    assert_eq!(me["role"], "member");
    assert_eq!(me["must_change_password"], true);
    assert_eq!(me["display_name"], "Ada");
    assert_eq!(me["has_avatar"], false);
}

#[tokio::test]
async fn me_without_a_principal_is_unauthorized() {
    let response = app()
        .oneshot(json_request("GET", "/identity/me", ""))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_password_without_origin_is_forbidden() {
    // No actor and no origin: the extractor answers 401 first, proving the
    // guard order (authentication before CSRF) to callers.
    let response = app()
        .oneshot(json_request(
            "POST",
            "/identity/me/password",
            r#"{"password":"new-long-pass"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_of_a_wrong_password_is_unauthorized_even_with_valid_shape() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let request = json_request(
        "POST",
        "/identity/login",
        r#"{"email":"member@example.com","password":"definitely-wrong"}"#,
    );

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_unparsable_credentials() {
    let router = seeded_app(vec![]);

    let response = router
        .oneshot(json_request(
            "POST",
            "/identity/login",
            r#"{"email":"not-an-email","password":"x"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
