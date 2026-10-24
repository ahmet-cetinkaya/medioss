use axum::http::StatusCode;

use crate::router_tests::*;

#[tokio::test]
async fn members_are_listed_in_program_order_with_progress() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        "/meditation/1/completed",
        r#"{"completed":true}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/programs/{program}/members"),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let names: Vec<_> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["M0", "M1", "M2"]);
    assert_eq!(json[0]["completed"], true);
    assert_eq!(json[1]["completed"], false);
}

#[tokio::test]
async fn member_routes_require_the_right_role() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/programs/1/members", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let body = r#"{"meditation_id":1}"#;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        "/meditation/programs/1/members",
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        "/meditation/programs/1/meditations/order",
        r#"{"meditation_ids":[]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "DELETE",
        "/meditation/programs/1/members/1",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) =
        json_body(&app, None, "DELETE", "/meditation/programs/1/members/1", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn add_answers_201_and_remove_answers_204() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (content_type, body) = multipart_body(&[("title", "", b"P")]);
    let (_, _, created) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs",
        vec![("content-type", &content_type)],
        axum::body::Body::from(body),
    )
    .await;
    let program = serde_json::from_slice::<serde_json::Value>(&created).unwrap()["id"]
        .as_i64()
        .unwrap();

    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "POST",
        &format!("/meditation/programs/{program}/members"),
        &format!(r#"{{"meditation_id":{}}}"#, meditation.id()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(member_ids(&app, program).await, vec![meditation.id()]);

    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "DELETE",
        &format!("/meditation/programs/{program}/members/{}", meditation.id()),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(member_ids(&app, program).await.is_empty());
}

#[tokio::test]
async fn adding_unknown_meditation_or_program_is_404() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/programs/999/members",
        &format!(r#"{{"meditation_id":{}}}"#, meditation.id()),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/meditation/programs/999/members/1",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reorder_echoes_program_and_ids() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    let (status, _, body) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        &format!("/meditation/programs/{program}/meditations/order"),
        r#"{"meditation_ids":[3,1,2]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"program_id": program, "meditation_ids": [3, 1, 2]})
    );
    assert_eq!(member_ids(&app, program).await, vec![3, 1, 2]);
}

#[tokio::test]
async fn reorder_with_a_partial_or_foreign_id_list_is_400() {
    let app = app();
    let program = seed_ordered_program(&app).await;
    let path = format!("/meditation/programs/{program}/meditations/order");
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        &path,
        r#"{"meditation_ids":[1,2]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        &path,
        r#"{"meditation_ids":[1,2,99]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(member_ids(&app, program).await, vec![1, 2, 3]);
}

#[tokio::test]
async fn state_changes_reject_foreign_origin() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/meditation/programs/1/members/1",
        vec![("origin", "http://evil.test")],
        axum::body::Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
