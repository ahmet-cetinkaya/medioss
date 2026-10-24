use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

const MIX: &str = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;

fn parse(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).unwrap()
}

async fn create_mix(app: &TestApp, payload: &str) -> (StatusCode, Vec<u8>) {
    let (headers, body) = json(payload);
    let (status, _, body) = call(
        app,
        Some(member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    (status, body)
}

#[tokio::test]
async fn created_mix_is_returned_by_get_with_its_items() {
    let app = app();
    let (status, body) = create_mix(&app, MIX).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = parse(&body)["id"].as_i64().unwrap();

    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/soundscape/mixes/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let mix = parse(&body);
    assert_eq!(mix["name"], "Focus");
    assert_eq!(mix["items"][0]["sound_id"], 1);
    assert_eq!(mix["items"][0]["volume"], 50);
}

#[tokio::test]
async fn invalid_mix_input_is_a_client_error() {
    let app = app();
    let (status, _) = create_mix(
        &app,
        r#"{"name":"Focus","items":[{"sound_id":1,"volume":500,"position":0}]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = create_mix(&app, r#"{"name":"Focus"}"#).await;
    assert!(status.is_client_error());
}

#[tokio::test]
async fn update_replaces_the_mix_and_unknown_ids_are_not_found() {
    let app = app();
    let (_, body) = create_mix(&app, MIX).await;
    let id = parse(&body)["id"].as_i64().unwrap();

    let (headers, body) =
        json(r#"{"name":"Sleep","items":[{"sound_id":1,"volume":20,"position":0}]}"#);
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "PUT",
        &format!("/soundscape/mixes/{id}"),
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse(&body)["name"], "Sleep");

    let (headers, body) = json(MIX);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "PUT",
        "/soundscape/mixes/99",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_removes_the_mix_for_its_owner_only() {
    let app = app();
    let (_, body) = create_mix(&app, MIX).await;
    let id = parse(&body)["id"].as_i64().unwrap();
    let path = format!("/soundscape/mixes/{id}");

    let (status, _, _) = call(
        &app,
        Some(another_member_actor()),
        "DELETE",
        &path,
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "DELETE",
        &path,
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &path,
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_contains_only_the_callers_mixes() {
    let app = app();
    create_mix(&app, MIX).await;
    let (status, _, body) = call(
        &app,
        Some(another_member_actor()),
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse(&body), serde_json::json!([]));
}
