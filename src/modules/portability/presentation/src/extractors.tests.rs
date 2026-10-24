use super::CurrentActor;
use crate::test_support::{actor, USER_ID};
use axum::extract::FromRequestParts;
use axum::http::{Request, StatusCode};

fn parts_with(actor: Option<kernel::Actor>) -> axum::http::request::Parts {
    let mut request = Request::builder().body(()).unwrap();
    if let Some(actor) = actor {
        request.extensions_mut().insert(actor);
    }
    request.into_parts().0
}

#[tokio::test]
async fn extracts_the_actor_inserted_by_the_session_middleware() {
    let mut parts = parts_with(Some(actor()));

    let CurrentActor(extracted) = CurrentActor::from_request_parts(&mut parts, &())
        .await
        .ok()
        .unwrap();

    assert_eq!(extracted.user_id().value(), USER_ID);
}

#[tokio::test]
async fn rejects_with_unauthorized_when_no_actor_is_present() {
    let mut parts = parts_with(None);

    let rejection = CurrentActor::from_request_parts(&mut parts, &())
        .await
        .err()
        .unwrap();

    assert_eq!(rejection, StatusCode::UNAUTHORIZED);
}
