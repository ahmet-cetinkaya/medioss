use chrono::{Duration, TimeZone, Utc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use kernel::UserId;
use tower::ServiceExt;

use meditation_application::{ports::SessionRepository, test_support::FakeSessionRepository};

use super::helpers::*;

#[tokio::test]
async fn statistics_returns_only_the_callers_sessions() {
    let test_app = app();
    let sessions = FakeSessionRepository(test_app.state.clone());
    let started_at = Utc
        .with_ymd_and_hms(2026, 9, 29, 8, 0, 0)
        .single()
        .expect("test instant");
    sessions
        .record_if_new(
            UserId::new(2),
            1,
            started_at,
            started_at + Duration::minutes(25),
        )
        .await
        .expect("member session");
    sessions
        .record_if_new(
            UserId::new(1),
            2,
            started_at,
            started_at + Duration::minutes(60),
        )
        .await
        .expect("admin session");

    let (status, _, body) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let body = String::from_utf8(body).expect("json");
    assert!(body.contains("\"total_minutes\":25"));
    assert!(!body.contains("60"));
}

#[tokio::test]
async fn statistics_returns_401_and_rejects_invalid_query_values() {
    let test_app = app();
    let response = test_app
        .router
        .clone()
        .oneshot(
            Request::get("/meditation/statistics?range=all&tz_offset_minutes=0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=year&tz_offset_minutes=0",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _, _) = call(
        &test_app,
        Some(member_actor()),
        "GET",
        "/meditation/statistics?range=all&tz_offset_minutes=1441",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
