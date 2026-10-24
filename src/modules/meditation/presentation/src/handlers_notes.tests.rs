use axum::http::StatusCode;

use crate::router_tests::*;

async fn post_note(
    app: &TestApp,
    actor: Option<kernel::Actor>,
    id: i64,
    text: &str,
) -> (StatusCode, serde_json::Value) {
    let (status, _, body) = json_body(
        app,
        actor,
        "POST",
        &format!("/meditation/{id}/notes"),
        &serde_json::json!({ "text": text }).to_string(),
    )
    .await;
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
async fn create_returns_201_with_note_shape() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, note) = post_note(&app, Some(member_actor()), meditation.id(), "calm").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(note["text"], "calm");
    assert!(note["id"].is_i64());
    assert!(note["created_at"].is_string());
    assert!(note["updated_at"].is_string());
}

#[tokio::test]
async fn list_returns_only_the_callers_notes() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    post_note(&app, Some(member_actor()), meditation.id(), "mine").await;
    post_note(&app, Some(admin_actor()), meditation.id(), "theirs").await;
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["text"], "mine");
}

#[tokio::test]
async fn notes_require_a_session() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", "/meditation/1/notes", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = post_note(&app, None, 1, "x").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn creating_on_an_unknown_meditation_is_404() {
    let app = app();
    let (status, _) = post_note(&app, Some(member_actor()), 999, "x").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn blank_text_is_a_bad_request() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _) = post_note(&app, Some(member_actor()), meditation.id(), "   ").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn editing_updates_the_text_and_answers_204() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (_, note) = post_note(&app, Some(member_actor()), meditation.id(), "old").await;
    let path = format!("/meditation/{}/notes/{}", meditation.id(), note["id"]);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        &path,
        r#"{"text":"new"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "",
    )
    .await;
    let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed[0]["id"], note["id"]);
    assert_eq!(listed[0]["text"], "new");
}

#[tokio::test]
async fn another_users_note_is_404_not_403() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (_, note) = post_note(&app, Some(member_actor()), meditation.id(), "private").await;
    let path = format!("/meditation/{}/notes/{}", meditation.id(), note["id"]);
    let (status, _, _) = json_body(
        &app,
        Some(admin_actor()),
        "PUT",
        &path,
        r#"{"text":"hijack"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = json_body(&app, Some(admin_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_answers_204_and_the_note_is_gone() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (_, note) = post_note(&app, Some(member_actor()), meditation.id(), "bye").await;
    let path = format!("/meditation/{}/notes/{}", meditation.id(), note["id"]);
    let (status, _, _) = json_body(&app, Some(member_actor()), "DELETE", &path, "").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "",
    )
    .await;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!([])
    );
}

#[tokio::test]
async fn mismatched_origin_is_forbidden() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/meditation/1/notes",
        vec![
            ("origin", "http://evil.test"),
            ("content-type", "application/json"),
        ],
        axum::body::Body::from(r#"{"text":"x"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
