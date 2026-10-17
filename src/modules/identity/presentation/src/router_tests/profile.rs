use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn profile_patch_sets_changes_and_clears_the_name() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let request = |body: &str| {
        let mut request = Request::patch("/identity/me/profile")
            .header("origin", "https://medioss.example")
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        request
            .extensions_mut()
            .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
        request
    };

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"  Ada  "}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(profile["display_name"], "Ada");

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"Ada L."}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = router
        .clone()
        .oneshot(request(r#"{"display_name":"   "}"#))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(profile["display_name"].is_null());
}

#[tokio::test]
async fn profile_patch_rejects_oversized_names_and_anonymous_calls() {
    let router = seeded_app(vec![user(1, "member@example.com", Role::Member)]);
    let long = "a".repeat(65);

    let mut request = Request::patch("/identity/me/profile")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "display_name": long }).to_string(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Member));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let request = Request::patch("/identity/me/profile")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"display_name":"Ada"}"#.to_owned()))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_and_users_list_expose_profile_fields() {
    let users = vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member)
            .with_display_name(Some(identity_domain::DisplayName::parse("Ada").unwrap()))
            .with_avatar(Some(identity_domain::AvatarRef::Webp)),
    ];
    let router = seeded_app(users);

    let mut request = Request::get("/identity/users").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed[1]["display_name"], "Ada");
    assert_eq!(listed[1]["has_avatar"], true);
    assert_eq!(listed[0]["display_name"], serde_json::Value::Null);
    assert_eq!(listed[0]["has_avatar"], false);
}
