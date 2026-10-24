use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

#[tokio::test]
async fn admin_updates_name_category_and_icon_key() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Storm","category_id":1,"icon_key":"thunder"}"#);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let sound: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sound["name"], "Storm");
    assert_eq!(sound["icon_key"], "thunder");
}

#[tokio::test]
async fn update_without_icon_key_clears_it() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Storm","category_id":1,"icon_key":"thunder"}"#);
    call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    let (headers, body) = json(r#"{"name":"Storm","category_id":1}"#);
    let (_, _, body) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    let sound: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sound["icon_key"], serde_json::Value::Null);
}

#[tokio::test]
async fn update_of_an_unknown_sound_is_not_found() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Storm","category_id":1}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/99",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_with_a_blank_name_is_a_bad_request() {
    let app = app();
    let (headers, body) = json(r#"{"name":"  ","category_id":1}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_removes_the_sound_once() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/sounds/1",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/sounds/1",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn members_cannot_update_or_delete() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Storm","category_id":1}"#);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "DELETE",
        "/soundscape/sounds/1",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
