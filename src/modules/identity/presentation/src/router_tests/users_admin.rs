use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use super::helpers::*;

#[tokio::test]
async fn users_list_returns_sorted_users_for_admin() {
    let router = seeded_app(vec![
        disabled_user(3, "mid@example.com", Role::Member),
        user(2, "zoe@example.com", Role::Member),
        user(1, "adam@example.com", Role::Admin),
    ]);
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
    let emails: Vec<&str> = listed
        .iter()
        .map(|u| u["email"].as_str().unwrap())
        .collect();
    assert_eq!(
        emails,
        vec!["adam@example.com", "mid@example.com", "zoe@example.com"]
    );
    let flags: Vec<bool> = listed
        .iter()
        .map(|u| u["disabled"].as_bool().unwrap())
        .collect();
    assert_eq!(flags, vec![false, true, false]);
}

#[tokio::test]
async fn users_list_forbids_members() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::get("/identity/users").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn users_list_requires_authentication() {
    let router = seeded_app(vec![]);
    let response = router
        .oneshot(Request::get("/identity/users").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_create_forbids_members_at_the_route_level() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::post("/identity/users")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"new@example.com","password":"long-enough"}"#,
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn user_status_disables_and_reenables_for_admin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::patch("/identity/users/2/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let mut request = Request::patch("/identity/users/2/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":false}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn user_status_refuses_last_active_admin() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn user_status_rejects_members_and_anonymous() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let request = Request::patch("/identity/users/1/status")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_delete_removes_member_for_admin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/users/2")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn user_delete_refuses_last_admin_without_purging() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::delete("/identity/users/1")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

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
    assert_eq!(listed.len(), 1, "refused delete must keep the user row");
}

#[tokio::test]
async fn me_delete_removes_own_account_and_flushes_session() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/me")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn me_delete_refuses_last_active_admin() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::delete("/identity/me")
        .header("origin", "https://medioss.example")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lifecycle_mutations_require_browser_origin() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);
    let mut request = Request::delete("/identity/users/2")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
