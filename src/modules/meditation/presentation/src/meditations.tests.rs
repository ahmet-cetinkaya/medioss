use axum::http::StatusCode;

use crate::router_tests::*;

#[tokio::test]
async fn daily_suggestion_is_204_when_library_is_empty() {
    let app = app();
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/daily-suggestion",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_empty());
}

#[tokio::test]
async fn daily_suggestion_returns_a_meditation_with_progress() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        "/meditation/daily-suggestion",
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["id"], meditation.id());
    assert_eq!(json["completed"], false);
    assert_eq!(json["progress_percent"], 0);
}

#[tokio::test]
async fn daily_suggestion_requires_a_session() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/daily-suggestion", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn library_query_longer_than_200_chars_is_400() {
    let app = app();
    let long = "a".repeat(201);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/library?q={long}"),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let exact = "a".repeat(200);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/library?q={exact}"),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn library_requires_a_session() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/library", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn library_attaches_the_callers_completion_state() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        &format!("/meditation/{}/completed", meditation.id()),
        r#"{"completed":true}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let listed = library_q(&app, "").await;
    assert_eq!(listed[0]["completed"], true);
}

#[tokio::test]
async fn library_names_every_program_the_meditation_belongs_to() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let program = seed_ordered_program(&app).await;
    // seed_ordered_program adds meditations 1..3 to program P; the seeded
    // meditation is a later id, so add it explicitly.
    use meditation_application::ports::MembershipRepository;
    use meditation_application::test_support::FakeMembershipRepository;
    FakeMembershipRepository(app.state.clone())
        .add(program, meditation.id(), chrono::Utc::now())
        .await
        .unwrap();
    let listed = library_q(&app, "").await;
    let entry = listed
        .iter()
        .find(|row| row["id"] == meditation.id())
        .expect("seeded meditation in the library");
    assert_eq!(
        entry["programs"],
        serde_json::json!([{ "id": program, "title": "P" }]),
    );
}

#[tokio::test]
async fn detail_returns_metadata_and_unknown_id_is_404() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Seed");
    assert_eq!(json["duration_ms"], 60_000);
    let (status, _, _) = json_body(&app, Some(member_actor()), "GET", "/meditation/999", "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_is_admin_only_and_removes_the_blob() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = format!("/meditation/{}", meditation.id());
    let (status, _, _) = json_body(&app, Some(member_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = json_body(&app, None, "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _, _) = json_body(&app, Some(admin_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = json_body(&app, Some(member_actor()), "GET", &path, "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = json_body(&app, Some(admin_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn description_update_returns_the_meditation_and_validates() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, json) = put_description(
        &app,
        Some(admin_actor()),
        meditation.id(),
        r#"{"description":"deep"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["description"], "deep");
    let too_long = serde_json::json!({ "description": "a".repeat(2001) }).to_string();
    let (status, _) = put_description(&app, Some(admin_actor()), meditation.id(), &too_long).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = put_description(
        &app,
        Some(member_actor()),
        meditation.id(),
        r#"{"description":"x"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) =
        put_description(&app, Some(admin_actor()), 999, r#"{"description":"x"}"#).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn description_update_rejects_foreign_origin() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/meditations/1",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", "application/json"),
        ],
        axum::body::Body::from(r#"{"description":"x"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

async fn put_name(
    app: &TestApp,
    actor: Option<kernel::Actor>,
    id: i64,
    body: &str,
) -> (StatusCode, serde_json::Value) {
    let (status, _, body) = json_body(
        app,
        actor,
        "PUT",
        &format!("/meditation/meditations/{id}/name"),
        body,
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
async fn rename_returns_the_meditation_and_validates() {
    let app = app();
    let meditation = seed_meditation(&app).await;

    let (status, json) = put_name(
        &app,
        Some(admin_actor()),
        meditation.id(),
        r#"{"name":"  Evening calm "}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "Evening calm");

    let (status, _) = put_name(
        &app,
        Some(admin_actor()),
        meditation.id(),
        r#"{"name":"   "}"#,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rename_is_admin_only_and_unknown_ids_are_404() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _) = put_name(
        &app,
        Some(member_actor()),
        meditation.id(),
        r#"{"name":"x"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = put_name(&app, None, meditation.id(), r#"{"name":"x"}"#).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = put_name(&app, Some(admin_actor()), 999, r#"{"name":"x"}"#).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rename_rejects_foreign_origin() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/meditation/meditations/1/name",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", "application/json"),
        ],
        axum::body::Body::from(r#"{"name":"x"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
