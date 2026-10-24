use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use identity_application::ApplicationError;
use kernel::{Actor, Role};

use super::{AdminActor, CurrentActor, CurrentPrincipal, Principal};

fn parts_with(value: Option<Actor>, principal: Option<Principal>) -> Parts {
    let mut request = axum::http::Request::builder()
        .body(())
        .unwrap();
    if let Some(actor) = value {
        request.extensions_mut().insert(actor);
    }
    if let Some(principal) = principal {
        request.extensions_mut().insert(principal);
    }
    let (parts, ()) = request.into_parts();
    parts
}

fn admin_actor() -> Actor {
    Actor::new(kernel::UserId::new(1), Role::Admin)
}

fn principal(role: Role) -> Principal {
    Principal {
        id: 2,
        email: "principal@medioss.test".to_owned(),
        role,
        must_change_password: false,
        auth_version: 4,
        display_name: Some("Ahmet".to_owned()),
        has_avatar: true,
    }
}

#[tokio::test]
async fn current_actor_reads_the_middleware_inserted_actor() {
    let mut parts = parts_with(Some(admin_actor()), None);

    let Ok(extracted) = CurrentActor::from_request_parts(&mut parts, &()).await else {
        panic!("an inserted actor must extract");
    };
    assert_eq!(extracted.0.user_id(), kernel::UserId::new(1));
    assert_eq!(extracted.0.role(), Role::Admin);
}

#[tokio::test]
async fn current_actor_without_an_actor_is_invalid_credentials() {
    let mut parts = parts_with(None, None);

    let rejection = CurrentActor::from_request_parts(&mut parts, &()).await;
    assert!(matches!(rejection, Err(error) if error.0 == ApplicationError::InvalidCredentials));
}

#[tokio::test]
async fn admin_actor_admits_admins_and_forbids_members() {
    let mut parts = parts_with(Some(admin_actor()), None);
    let Ok(extracted) = AdminActor::from_request_parts(&mut parts, &()).await else {
        panic!("an inserted admin actor must extract");
    };
    assert_eq!(extracted.0.user_id(), kernel::UserId::new(1));

    let mut parts = parts_with(Some(Actor::new(kernel::UserId::new(3), Role::Member)), None);
    let rejection = AdminActor::from_request_parts(&mut parts, &()).await;
    assert!(matches!(rejection, Err(error) if error.0 == ApplicationError::Forbidden));
}

#[tokio::test]
async fn admin_actor_without_any_actor_is_invalid_credentials() {
    let mut parts = parts_with(None, None);

    let rejection = AdminActor::from_request_parts(&mut parts, &()).await;
    assert!(matches!(rejection, Err(error) if error.0 == ApplicationError::InvalidCredentials));
}

#[tokio::test]
async fn current_principal_reads_the_session_principal_snapshot() {
    let mut parts = parts_with(None, Some(principal(Role::Member)));

    let Ok(extracted) = CurrentPrincipal::from_request_parts(&mut parts, &()).await else {
        panic!("an inserted principal must extract");
    };
    assert_eq!(extracted.0.id, 2);
    assert_eq!(extracted.0.email, "principal@medioss.test");
    assert_eq!(extracted.0.auth_version, 4);
    assert_eq!(extracted.0.display_name, Some("Ahmet".to_owned()));
    assert!(extracted.0.has_avatar);
}

#[tokio::test]
async fn current_principal_without_a_principal_is_invalid_credentials() {
    let mut parts = parts_with(None, None);

    let rejection = CurrentPrincipal::from_request_parts(&mut parts, &()).await;
    assert!(matches!(rejection, Err(error) if error.0 == ApplicationError::InvalidCredentials));
    let _ = parts_with(None, None);
}
