use axum::{body::Body, http::Request, http::StatusCode, routing::get, Router};
use kernel::{Actor, Role, UserId};
use tower::ServiceExt;

use super::{admin, AdminActor, CurrentActor};

async fn current(CurrentActor(actor): CurrentActor) -> String {
    actor.user_id().value().to_string()
}

async fn admin_only(AdminActor(actor): AdminActor) -> String {
    actor.user_id().value().to_string()
}

async fn status_and_body(actor: Option<Actor>, path: &str) -> (StatusCode, String) {
    let router = Router::new()
        .route("/current", get(current))
        .route("/admin", get(admin_only));
    let mut request = Request::builder().uri(path).body(Body::empty()).unwrap();
    if let Some(actor) = actor {
        request.extensions_mut().insert(actor);
    }
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn actor(id: i64, role: Role) -> Actor {
    Actor::new(UserId::new(id), role)
}

#[tokio::test]
async fn current_actor_requires_an_actor_in_the_request() {
    assert_eq!(
        status_and_body(None, "/current").await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn current_actor_accepts_members_and_admins() {
    assert_eq!(
        status_and_body(Some(actor(2, Role::Member)), "/current").await,
        (StatusCode::OK, "2".to_owned())
    );
    assert_eq!(
        status_and_body(Some(actor(1, Role::Admin)), "/current").await,
        (StatusCode::OK, "1".to_owned())
    );
}

#[tokio::test]
async fn admin_actor_is_unauthorized_without_actor_and_forbidden_for_members() {
    assert_eq!(
        status_and_body(None, "/admin").await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_and_body(Some(actor(2, Role::Member)), "/admin")
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn admin_actor_accepts_admins() {
    assert_eq!(
        status_and_body(Some(actor(1, Role::Admin)), "/admin").await,
        (StatusCode::OK, "1".to_owned())
    );
}

#[test]
fn admin_check_allows_only_admins() {
    assert_eq!(admin(&actor(1, Role::Admin)), Ok(()));
    assert_eq!(admin(&actor(2, Role::Member)), Err(StatusCode::FORBIDDEN));
}
