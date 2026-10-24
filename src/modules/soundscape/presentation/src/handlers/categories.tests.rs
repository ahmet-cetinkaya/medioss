use axum::{body::Body, http::StatusCode};

use crate::test_support::*;

fn parse(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).unwrap()
}

#[tokio::test]
async fn list_returns_categories_for_any_signed_in_actor() {
    let app = app();
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/categories",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        parse(&body),
        serde_json::json!([{"id": 1, "name": "Nature", "slug": "nature"}])
    );
}

#[tokio::test]
async fn admin_creates_a_category_and_a_duplicate_conflicts() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Ocean"}"#);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(parse(&body)["name"], "Ocean");

    let (headers, body) = json(r#"{"name":"Ocean"}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_with_a_blank_name_is_a_bad_request() {
    let app = app();
    let (headers, body) = json(r#"{"name":"   "}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rename_updates_the_category_or_reports_not_found() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Forest"}"#);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/categories/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse(&body)["name"], "Forest");

    let (headers, body) = json(r#"{"name":"Forest"}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/categories/99",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_distinguishes_in_use_missing_and_empty_categories() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/categories/1",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "category 1 holds the Rain sound"
    );

    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/categories/99",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (headers, body) = json(r#"{"name":"Empty"}"#);
    let (_, _, created) = call(
        &app,
        Some(admin_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    let id = parse(&created)["id"].as_i64().unwrap();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        &format!("/soundscape/categories/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn members_cannot_mutate_categories() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Nope"}"#);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "DELETE",
        "/soundscape/categories/1",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
