use axum::{body::Body, http::StatusCode};

use super::helpers::*;

#[tokio::test]
async fn members_manage_their_own_mixes() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );

    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("\"name\":\"Focus\""),
        "member sees own mix: {text}"
    );
}

#[tokio::test]
async fn mix_of_another_user_is_invisible_and_untouchable() {
    let app = app();
    let payload = r#"{"name":"Mine","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, created) = call(
        &app,
        Some(another_member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: i64 = serde_json::from_slice::<serde_json::Value>(&created).unwrap()["id"]
        .as_i64()
        .unwrap();

    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/soundscape/mixes/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "foreign mix by id is 404");
}
