use axum::http::StatusCode;
use kernel::{Actor, Role, UserId};

use super::helpers::*;

#[tokio::test]
async fn completion_retry_is_idempotent_and_toggle_touches_nothing() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let started = chrono::Utc::now().to_rfc3339();
    let finished = chrono::Utc::now().to_rfc3339();
    let payload = format!("{{\"started_at\":\"{started}\",\"finished_at\":\"{finished}\"}}");

    let (first, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        &payload,
    )
    .await;
    assert_eq!(first, StatusCode::CREATED);
    let (retry, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        &payload,
    )
    .await;
    assert_eq!(retry, StatusCode::CREATED);
    assert_eq!(
        app.state.sessions.lock().unwrap().len(),
        1,
        "one session, not two"
    );

    // Toggle completed off and on: session count stays one.
    let id = meditation.id();
    for completed in [false, true] {
        let (status, _, _) = json_body(
            &app,
            Some(member_actor()),
            "PUT",
            &format!("/meditation/{id}/completed"),
            &format!("{{\"completed\":{completed}}}"),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert_eq!(app.state.sessions.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn notes_of_one_user_are_invisible_to_another() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/notes", meditation.id()),
        "{\"text\":\"private reflection\"}",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // The same member sees it; a different member's list is empty.
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "{}",
    )
    .await;
    assert!(String::from_utf8_lossy(&body).contains("private reflection"));

    let other = Actor::new(UserId::new(3), Role::Member);
    let (_, _, body) = json_body(
        &app,
        Some(other.clone()),
        "GET",
        &format!("/meditation/{}/notes", meditation.id()),
        "{}",
    )
    .await;
    assert_eq!(body, b"[]");

    // The other member cannot edit or delete what it cannot see.
    let (status, _, _) = json_body(
        &app,
        Some(other),
        "PUT",
        &format!("/meditation/{}/notes/1", meditation.id()),
        "{\"text\":\"hijack\"}",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn progress_percent_reflects_the_reported_position() {
    let app = app();
    let meditation = seed_meditation(&app).await; // duration_ms == 60_000

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    assert!(
        String::from_utf8_lossy(&body).contains("\"progress_percent\":0"),
        "untouched meditation reports 0, never omitted: {}",
        String::from_utf8_lossy(&body)
    );

    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/playback-state", meditation.id()),
        "{\"position_ms\":30000}",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("\"progress_percent\":50"),
        "30s of 60s must report 50%: {text}"
    );
    assert!(text.contains("\"completed\":false"));

    // Someone else's position report must not move this meditation's own
    // percent for the original caller.
    let other = Actor::new(UserId::new(3), Role::Member);
    let (_, _, body) = json_body(
        &app,
        Some(other),
        "GET",
        &format!("/meditation/{}", meditation.id()),
        "{}",
    )
    .await;
    assert!(String::from_utf8_lossy(&body).contains("\"progress_percent\":0"));
}
