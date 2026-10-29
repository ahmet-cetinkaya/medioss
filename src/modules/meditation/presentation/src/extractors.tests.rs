use axum::{extract::FromRequestParts, http::Request};
use kernel::{Actor, Role, UserId};

use super::{AdminActor, CurrentActor};
use crate::response::ApiError;

fn parts_with(actor: Option<Actor>) -> axum::http::request::Parts {
    let mut request = Request::builder().body(()).unwrap();
    if let Some(actor) = actor {
        request.extensions_mut().insert(actor);
    }
    request.into_parts().0
}

fn status_of(error: ApiError) -> axum::http::StatusCode {
    match error {
        ApiError::StatusCode(status) => status,
        ApiError::Application(_) | ApiError::InsufficientStorage => {
            panic!("extractors only reject with a bare status")
        }
    }
}

#[tokio::test]
async fn current_actor_yields_the_inserted_actor() {
    let actor = Actor::new(UserId::new(7), Role::Member);
    let mut parts = parts_with(Some(actor.clone()));
    let CurrentActor(extracted) = CurrentActor::from_request_parts(&mut parts, &())
        .await
        .ok()
        .expect("actor present");
    assert_eq!(extracted, actor);
}

#[tokio::test]
async fn current_actor_without_session_is_unauthorized() {
    let mut parts = parts_with(None);
    let error = CurrentActor::from_request_parts(&mut parts, &())
        .await
        .err()
        .expect("rejected");
    assert_eq!(status_of(error), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_actor_accepts_an_admin() {
    let actor = Actor::new(UserId::new(1), Role::Admin);
    let mut parts = parts_with(Some(actor.clone()));
    let AdminActor(extracted) = AdminActor::from_request_parts(&mut parts, &())
        .await
        .ok()
        .expect("admin accepted");
    assert_eq!(extracted, actor);
}

#[tokio::test]
async fn admin_actor_rejects_a_member_as_forbidden() {
    let mut parts = parts_with(Some(Actor::new(UserId::new(2), Role::Member)));
    let error = AdminActor::from_request_parts(&mut parts, &())
        .await
        .err()
        .expect("rejected");
    assert_eq!(status_of(error), axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_actor_without_session_is_unauthorized_not_forbidden() {
    let mut parts = parts_with(None);
    let error = AdminActor::from_request_parts(&mut parts, &())
        .await
        .err()
        .expect("rejected");
    assert_eq!(status_of(error), axum::http::StatusCode::UNAUTHORIZED);
}
