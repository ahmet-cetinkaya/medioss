use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::Role;
use tower::ServiceExt;

use crate::router_tests::helpers::*;
use crate::users_lifecycle::UserDeletionListener;

fn actor_request(method: &str, path: &str, role: Role, disabled: bool) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"disabled":{disabled}}}"#)))
        .unwrap();
    request.extensions_mut().insert(kernel::Actor::new(kernel::UserId::new(1), role));
    request
}

#[tokio::test]
async fn status_changes_require_an_admin_actor() {
    let router = seeded_app(vec![
        user(1, "admin@example.com", Role::Admin),
        user(2, "member@example.com", Role::Member),
    ]);

    let member_response = router
        .clone()
        .oneshot(actor_request("PATCH", "/identity/users/2/status", Role::Member, true))
        .await
        .unwrap();
    assert_eq!(member_response.status(), StatusCode::FORBIDDEN);

    let admin_response = router
        .oneshot(actor_request("PATCH", "/identity/users/2/status", Role::Admin, true))
        .await
        .unwrap();
    assert_eq!(admin_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn status_change_without_origin_is_forbidden() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::patch("/identity/users/2/status")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":true}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn self_delete_without_origin_is_forbidden() {
    let router = seeded_app(vec![user(2, "member@example.com", Role::Member)]);
    let mut request = Request::delete("/identity/me").body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(2), Role::Member));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_delete_of_the_last_active_admin_is_rejected() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);
    let mut request = Request::delete("/identity/users/1")
        .header("origin", "https://medioss.example")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"disabled":false}"#))
        .unwrap();
    request
        .extensions_mut()
        .insert(kernel::Actor::new(kernel::UserId::new(1), Role::Admin));

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn noop_deletion_listener_confirms_nothing_to_do() {
    let listener = crate::users_lifecycle::NoUserDeletion;

    assert_eq!(listener.on_user_deleted(kernel::UserId::new(3)).await, Ok(()));
}

#[tokio::test]
async fn status_change_for_an_unknown_user_answers_unauthorized() {
    let router = seeded_app(vec![user(1, "admin@example.com", Role::Admin)]);

    let response = router
        .oneshot(actor_request(
            "PATCH",
            "/identity/users/999/status",
            Role::Admin,
            true,
        ))
        .await
        .unwrap();

    // Unknown targets answer with the same uniform 401 as bad credentials.
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
