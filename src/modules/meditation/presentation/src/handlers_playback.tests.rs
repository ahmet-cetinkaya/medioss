use axum::http::StatusCode;

use crate::router_tests::*;

fn playback_path(id: i64) -> String {
    format!("/meditation/{id}/playback-state")
}

#[tokio::test]
async fn untouched_meditation_reports_zero_and_not_completed() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &playback_path(meditation.id()),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let state: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        state,
        serde_json::json!({"position_ms": 0, "completed": false})
    );
}

#[tokio::test]
async fn recorded_position_round_trips_per_user() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let path = playback_path(meditation.id());
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &path,
        r#"{"position_ms":12000}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, _, body) = json_body(&app, Some(member_actor()), "GET", &path, "").await;
    let mine: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(mine["position_ms"], 12000);

    let (_, _, body) = json_body(&app, Some(admin_actor()), "GET", &path, "").await;
    let other: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(other["position_ms"], 0);
}

#[tokio::test]
async fn playback_routes_require_a_session() {
    let app = app();
    let (status, _, _) = json_body(&app, None, "GET", &playback_path(1), "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = json_body(
        &app,
        None,
        "POST",
        &playback_path(1),
        r#"{"position_ms":1}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = json_body(
        &app,
        None,
        "PUT",
        "/meditation/1/completed",
        r#"{"completed":true}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unknown_meditation_is_404() {
    let app = app();
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &playback_path(999),
        r#"{"position_ms":1}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "PUT",
        "/meditation/999/completed",
        r#"{"completed":true}"#,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn malformed_position_body_is_rejected_as_client_error() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &playback_path(meditation.id()),
        r#"{"position_ms":"soon"}"#,
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
}

#[tokio::test]
async fn set_completed_toggles_the_flag() {
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
    let (_, _, body) = json_body(
        &app,
        Some(member_actor()),
        "GET",
        &playback_path(meditation.id()),
        "",
    )
    .await;
    let state: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(state["completed"], true);
}

#[tokio::test]
async fn completion_creates_a_session_with_rfc3339_times() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, body) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        r#"{"started_at":"2026-01-01T10:00:00Z","finished_at":"2026-01-01T10:10:00Z"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let session: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(session["session_id"].is_i64());
    assert_eq!(session["started_at"], "2026-01-01T10:00:00+00:00");
    assert_eq!(session["finished_at"], "2026-01-01T10:10:00+00:00");
}

#[tokio::test]
async fn completion_with_non_timestamp_body_is_a_client_error() {
    let app = app();
    let meditation = seed_meditation(&app).await;
    let (status, _, _) = json_body(
        &app,
        Some(member_actor()),
        "POST",
        &format!("/meditation/{}/completion", meditation.id()),
        r#"{"started_at":"yesterday","finished_at":"today"}"#,
    )
    .await;
    assert!(status.is_client_error(), "got {status}");
}

#[tokio::test]
async fn state_changes_reject_a_foreign_origin() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        &playback_path(1),
        vec![
            ("origin", "http://evil.test"),
            ("content-type", "application/json"),
        ],
        axum::body::Body::from(r#"{"position_ms":1}"#),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
