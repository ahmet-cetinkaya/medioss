use axum::http::StatusCode;

use crate::router_tests::*;

async fn create(app: &TestApp, name: &str) -> (StatusCode, serde_json::Value) {
    let (status, _, body) = json_body(
        app,
        Some(admin_actor()),
        "POST",
        "/meditation/categories",
        &serde_json::json!({ "name": name }).to_string(),
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
async fn create_returns_201_with_id_and_name_and_list_shows_it() {
    let app = app();
    let (status, created) = create(&app, "Sleep").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Sleep");

    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/categories",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        listed,
        serde_json::json!([{ "id": created["id"], "name": "Sleep" }])
    );
}

#[tokio::test]
async fn list_requires_a_session() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/categories", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn duplicate_name_conflicts() {
    let app = app();
    assert_eq!(create(&app, "Focus").await.0, StatusCode::CREATED);
    let (status, body) = create(&app, "Focus").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn blank_name_is_a_bad_request() {
    let app = app();
    assert_eq!(create(&app, "   ").await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn management_is_admin_only() {
    let app = app();
    let body = r#"{"name":"X"}"#;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        "/meditation/categories",
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(&app, None, "POST", "/meditation/categories", body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        "/meditation/categories/1",
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "DELETE",
        "/meditation/categories/1",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn mismatched_origin_is_forbidden() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/meditation/categories",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", "application/json"),
        ],
        axum::body::Body::from(r#"{"name":"X"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn rename_answers_204_and_unknown_id_is_404() {
    let app = app();
    let (_, created) = create(&app, "Old").await;
    let path = format!("/meditation/categories/{}", created["id"]);
    let (status, _, _) =
        json_body(&app, Some(admin_actor()), "PUT", &path, r#"{"name":"New"}"#).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/categories",
        "",
    )
    .await;
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed[0]["name"], "New");

    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/categories/999",
        r#"{"name":"Z"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_answers_204_and_removes_it() {
    let app = app();
    let (_, created) = create(&app, "Gone").await;
    let path = format!("/meditation/categories/{}", created["id"]);
    let (status, _, _) = json_body(&app, Some(admin_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/categories",
        "",
    )
    .await;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!([])
    );
}

#[tokio::test]
async fn assigning_an_unknown_program_is_404() {
    let app = app();
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/programs/999/category",
        r#"{"category_id":null}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
