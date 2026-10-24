use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use kernel::{Actor, Role, UserId};
use std::sync::Arc as StdArc;

use mood_tracker_application::{
    commands::create_mood_entry::CreateMoodEntryService,
    commands::delete_mood_entry::DeleteMoodEntryService,
    commands::update_mood_entry::UpdateMoodEntryService,
    ports::{
        Clock, MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
    },
    queries::get_mood_history::GetMoodHistoryService,
    queries::get_mood_statistics::GetMoodStatisticsService,
    queries::get_timeline::GetTimelineService,
    test_support::{FakeMoodEntryRepository, FixedClock},
};
use serde::Deserialize;
use tower::ServiceExt;

use super::mood_tracker_router;
use crate::MoodTrackerDependencies;

#[derive(Clone)]
struct FakeMeditationTimelineSource(Vec<MeditationTimelineEvent>);

#[async_trait]
impl MeditationTimelineSource for FakeMeditationTimelineSource {
    async fn list_for_user(
        &self,
        _user_id: UserId,
    ) -> Result<Vec<MeditationTimelineEvent>, String> {
        Ok(self.0.clone())
    }
}

struct TestApp {
    router: Router,
}

fn app(events: Vec<MeditationTimelineEvent>) -> TestApp {
    let entries: StdArc<dyn mood_tracker_application::ports::MoodEntryRepository> =
        StdArc::new(FakeMoodEntryRepository::default());
    let clock: StdArc<dyn Clock> = StdArc::new(FixedClock(
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0)
            .single()
            .expect("test instant"),
    ));
    let timeline_entries = entries.clone();
    let deps = MoodTrackerDependencies {
        create: StdArc::new(CreateMoodEntryService::new(entries.clone(), clock.clone())),
        update: StdArc::new(UpdateMoodEntryService::new(entries.clone(), clock.clone())),
        delete: StdArc::new(DeleteMoodEntryService::new(entries.clone())),
        get_mood_history: StdArc::new(GetMoodHistoryService::new(entries.clone(), clock.clone())),
        get_mood_statistics: StdArc::new(GetMoodStatisticsService::new(entries, clock.clone())),
        get_timeline: StdArc::new(GetTimelineService::new(
            timeline_entries,
            StdArc::new(FakeMeditationTimelineSource(events)),
            clock,
        )),
        medioss_origin: Arc::new("http://localhost".to_owned()),
    };
    TestApp {
        router: mood_tracker_router().with_state(Arc::new(deps)),
    }
}

fn member_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Member)
}

fn event(id: i64, recorded_at: DateTime<Utc>) -> MeditationTimelineEvent {
    MeditationTimelineEvent {
        id,
        meditation_name: format!("Meditation {id}"),
        note: None,
        recorded_at,
        kind: MeditationTimelineEventKind::CompletedSession,
    }
}

#[derive(Deserialize)]
struct TimelinePage {
    items: Vec<TimelineItem>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
struct TimelineItem {
    id: i64,
}

async fn timeline_page(app: &TestApp, path: &str) -> (StatusCode, TimelinePage) {
    let mut request = Request::get(path).body(Body::empty()).expect("request");
    request.extensions_mut().insert(member_actor());
    let response = app.router.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
        .await
        .expect("body");
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    (
        status,
        serde_json::from_slice(&body).expect("timeline JSON"),
    )
}

#[tokio::test]
async fn timeline_cursor_continues_descending_without_duplicates() {
    let latest = Utc
        .with_ymd_and_hms(2026, 9, 30, 12, 0, 0)
        .single()
        .expect("test instant");
    let app = app((1..=52)
        .map(|id| {
            event(
                id,
                if id <= 50 {
                    latest - Duration::seconds(id)
                } else {
                    latest - Duration::seconds(50)
                },
            )
        })
        .collect());

    let (status, first) = timeline_page(&app, "/timeline?limit=50").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first.items.len(), 50);
    assert_eq!(first.items.last().expect("last item").id, 52);
    let cursor = first.next_cursor.expect("first page cursor");

    let (status, second) = timeline_page(
        &app,
        &format!("/timeline?limit=50&cursor={}", cursor.replace('+', "%2B")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(second.items.len(), 2);
    assert_eq!(
        second.items.iter().map(|item| item.id).collect::<Vec<_>>(),
        [51, 50]
    );
    assert!(second.items.iter().all(|item| !first
        .items
        .iter()
        .any(|first_item| first_item.id == item.id)));
}

async fn status_of(app: &TestApp, path: &str) -> StatusCode {
    let mut request = Request::get(path).body(Body::empty()).expect("request");
    request.extensions_mut().insert(member_actor());
    app.router
        .clone()
        .oneshot(request)
        .await
        .expect("response")
        .status()
}

fn at_noon(month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, month, day, 12, 0, 0)
        .single()
        .expect("test instant")
}

#[tokio::test]
async fn custom_range_returns_only_events_inside_both_inclusive_dates() {
    let app = app(vec![
        event(1, at_noon(9, 9)),
        event(2, at_noon(9, 10)),
        event(3, at_noon(9, 20)),
        event(4, at_noon(9, 21)),
    ]);

    let (_, page) = timeline_page(&app, "/timeline?from=2026-09-10&to=2026-09-20").await;

    let mut ids: Vec<_> = page.items.iter().map(|item| item.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, [2, 3], "first and last date count, neighbours do not");
}

#[tokio::test]
async fn custom_range_is_validated_with_a_400_for_every_bad_shape() {
    let app = app(Vec::new());
    for (name, path) in [
        ("from after to", "/timeline?from=2026-09-20&to=2026-09-10"),
        (
            "more than 366 days",
            "/timeline?from=2025-01-01&to=2026-09-30",
        ),
        ("only from", "/timeline?from=2026-09-10"),
        ("only to", "/timeline?to=2026-09-10"),
        ("not a date", "/timeline?from=yesterday&to=today"),
        ("impossible date", "/timeline?from=2026-02-30&to=2026-03-02"),
        (
            "preset mixed with custom",
            "/timeline?range=last7&from=2026-09-10&to=2026-09-20",
        ),
        ("history endpoint", "/mood?from=2026-09-20&to=2026-09-10"),
        (
            "statistics endpoint",
            "/mood/statistics?from=2026-09-20&to=2026-09-10",
        ),
    ] {
        assert_eq!(
            status_of(&app, path).await,
            StatusCode::BAD_REQUEST,
            "{name}"
        );
    }
}

#[tokio::test]
async fn presets_still_work_and_a_366_day_span_is_the_longest_custom_range() {
    let app = app(Vec::new());
    for path in [
        "/timeline",
        "/timeline?range=last7",
        "/mood/statistics?range=last30",
        "/timeline?from=2025-09-30&to=2026-09-30",
    ] {
        assert_eq!(status_of(&app, path).await, StatusCode::OK, "{path}");
    }
}

const ORIGIN: &str = "http://localhost";
const OTHER_USER: i64 = 2;

struct Call<'a> {
    method: &'a str,
    path: String,
    origin: Option<&'a str>,
    actor: Option<Actor>,
    body: Option<serde_json::Value>,
}

fn call<'a>(method: &'a str, path: &str) -> Call<'a> {
    Call {
        method,
        path: path.to_owned(),
        origin: Some(ORIGIN),
        actor: Some(member_actor()),
        body: None,
    }
}

async fn send(app: &TestApp, call: Call<'_>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder().method(call.method).uri(&call.path);
    if let Some(origin) = call.origin {
        builder = builder.header("origin", origin);
    }
    let body = match call.body {
        Some(json) => {
            builder = builder.header("content-type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let mut request = builder.body(body).expect("request");
    if let Some(actor) = call.actor {
        request.extensions_mut().insert(actor);
    }
    let response = app.router.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 128 * 1024)
        .await
        .expect("body");
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn create_entry(app: &TestApp, actor: Actor, score: u8, note: Option<&str>) -> i64 {
    let mut request = call("POST", "/mood");
    request.actor = Some(actor);
    request.body = Some(serde_json::json!({ "score": score, "note": note }));
    let (status, json) = send(app, request).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    json["id"].as_i64().expect("created id")
}

fn other_actor() -> Actor {
    Actor::new(UserId::new(OTHER_USER), Role::Member)
}

#[tokio::test]
async fn create_returns_the_stored_entry() {
    let app = app(Vec::new());
    let mut request = call("POST", "/mood");
    request.body = Some(serde_json::json!({ "score": 4, "note": "  calm  " }));

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["score"], 4);
    assert_eq!(json["note"], "calm");
    assert_eq!(json["recorded_at"], "2026-10-01T00:00:00+00:00");
}

#[tokio::test]
async fn create_rejects_an_out_of_range_score_with_400() {
    let app = app(Vec::new());
    let mut request = call("POST", "/mood");
    request.body = Some(serde_json::json!({ "score": 6 }));

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        json["error"],
        "input is invalid: mood score must be between 1 and 5"
    );
}

#[tokio::test]
async fn state_changing_routes_require_the_configured_origin() {
    let app = app(Vec::new());
    let id = create_entry(&app, member_actor(), 3, None).await;
    let body = serde_json::json!({ "score": 3 });
    let cases = [
        ("POST", "/mood".to_owned()),
        ("PUT", format!("/mood/{id}")),
        ("DELETE", format!("/mood/{id}")),
    ];
    for (method, path) in &cases {
        for origin in [None, Some("http://evil.example")] {
            let mut request = call(method, path);
            request.origin = origin;
            request.body = Some(body.clone());

            let (status, _) = send(&app, request).await;

            assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path} {origin:?}");
        }
    }
    let (_, history) = send(&app, call("GET", "/mood")).await;
    assert_eq!(history["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn origin_is_compared_case_insensitively() {
    let app = app(Vec::new());
    let mut request = call("POST", "/mood");
    request.origin = Some("HTTP://LOCALHOST");
    request.body = Some(serde_json::json!({ "score": 2 }));

    let (status, _) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn every_route_without_an_actor_is_401() {
    let app = app(Vec::new());
    for (method, path) in [
        ("GET", "/mood"),
        ("POST", "/mood"),
        ("PUT", "/mood/1"),
        ("DELETE", "/mood/1"),
        ("GET", "/mood/statistics"),
        ("GET", "/timeline"),
    ] {
        let mut request = call(method, path);
        request.actor = None;
        request.body = Some(serde_json::json!({ "score": 3 }));

        let (status, _) = send(&app, request).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}

#[tokio::test]
async fn update_changes_the_entry_and_keeps_its_recorded_time() {
    let app = app(Vec::new());
    let id = create_entry(&app, member_actor(), 2, Some("before")).await;
    let mut request = call("PUT", &format!("/mood/{id}"));
    request.body = Some(serde_json::json!({ "score": 5, "note": "after" }));

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["id"], id);
    assert_eq!(json["score"], 5);
    assert_eq!(json["note"], "after");
}

#[tokio::test]
async fn update_and_delete_of_another_users_entry_are_404_and_leave_it_intact() {
    let app = app(Vec::new());
    let id = create_entry(&app, member_actor(), 2, None).await;

    let mut update = call("PUT", &format!("/mood/{id}"));
    update.actor = Some(other_actor());
    update.body = Some(serde_json::json!({ "score": 5 }));
    let (update_status, _) = send(&app, update).await;
    let mut delete = call("DELETE", &format!("/mood/{id}"));
    delete.actor = Some(other_actor());
    let (delete_status, _) = send(&app, delete).await;

    assert_eq!(update_status, StatusCode::NOT_FOUND);
    assert_eq!(delete_status, StatusCode::NOT_FOUND);
    let (_, history) = send(&app, call("GET", "/mood")).await;
    assert_eq!(history["items"][0]["score"], 2);
}

#[tokio::test]
async fn delete_removes_the_entry_and_a_repeat_is_404() {
    let app = app(Vec::new());
    let id = create_entry(&app, member_actor(), 3, None).await;

    let (first, _) = send(&app, call("DELETE", &format!("/mood/{id}"))).await;
    let (second, _) = send(&app, call("DELETE", &format!("/mood/{id}"))).await;

    assert_eq!(first, StatusCode::OK);
    assert_eq!(second, StatusCode::NOT_FOUND);
    let (_, history) = send(&app, call("GET", "/mood")).await;
    assert!(history["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn history_is_empty_for_a_new_user_and_scoped_to_the_caller() {
    let app = app(Vec::new());
    create_entry(&app, other_actor(), 4, None).await;

    let (status, json) = send(&app, call("GET", "/mood")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["items"].as_array().unwrap().is_empty());
    assert!(json["next_cursor"].is_null());
}

#[tokio::test]
async fn history_pages_with_an_opaque_cursor() {
    let app = app(Vec::new());
    for score in 1..=3 {
        create_entry(&app, member_actor(), score, None).await;
    }

    let (_, first) = send(&app, call("GET", "/mood?limit=2")).await;
    let cursor = first["next_cursor"]
        .as_str()
        .expect("cursor")
        .replace('+', "%2B");
    let (_, second) = send(&app, call("GET", &format!("/mood?limit=2&cursor={cursor}"))).await;

    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    assert_eq!(second["items"].as_array().unwrap().len(), 1);
    assert!(second["next_cursor"].is_null());
}

#[tokio::test]
async fn invalid_query_values_are_400() {
    let app = app(Vec::new());
    for path in [
        "/mood?range=forever",
        "/mood?cursor=garbage",
        "/mood?cursor=2026-10-01T00:00:00%2B00:00:abc",
        "/mood?tz_offset_minutes=1440",
        "/mood?tz_offset_minutes=-1440",
        "/mood?limit=0",
        "/mood?limit=101",
        "/mood/statistics?range=forever",
        "/mood/statistics?tz_offset_minutes=1440",
        "/timeline?cursor=garbage",
        "/timeline?cursor=2026-10-01T00:00:00%2B00:00:Bogus:1",
        "/timeline?limit=0",
    ] {
        assert_eq!(
            status_of(&app, path).await,
            StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
}

#[tokio::test]
async fn offsets_at_the_inclusive_bounds_are_accepted() {
    let app = app(Vec::new());
    for path in [
        "/mood?tz_offset_minutes=1439",
        "/mood?tz_offset_minutes=-1439",
    ] {
        assert_eq!(status_of(&app, path).await, StatusCode::OK, "{path}");
    }
}

#[tokio::test]
async fn statistics_of_no_entries_are_zeroed() {
    let app = app(Vec::new());

    let (status, json) = send(&app, call("GET", "/mood/statistics")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["entry_count"], 0);
    assert!(json["average_score"].is_null());
    assert_eq!(json["distribution"], serde_json::json!([0, 0, 0, 0, 0]));
    assert!(json["trend"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn statistics_report_average_distribution_and_trend() {
    let app = app(Vec::new());
    create_entry(&app, member_actor(), 2, None).await;
    create_entry(&app, member_actor(), 4, None).await;
    create_entry(&app, other_actor(), 5, None).await;

    let (_, json) = send(&app, call("GET", "/mood/statistics")).await;

    assert_eq!(json["entry_count"], 2);
    assert_eq!(json["average_score"], 3.0);
    assert_eq!(json["distribution"], serde_json::json!([0, 1, 0, 1, 0]));
    assert_eq!(json["trend"][0]["date"], "2026-10-01");
    assert_eq!(json["trend"][0]["average_score"], 3.0);
}

#[tokio::test]
async fn timeline_merges_mood_entries_and_meditation_events() {
    let app = app(vec![event(9, at_noon(9, 20))]);
    create_entry(&app, member_actor(), 4, Some("note")).await;

    let (status, json) = send(&app, call("GET", "/timeline")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["items"][0]["kind"], "Mood");
    assert_eq!(json["items"][0]["title"], "Mood entry");
    assert_eq!(json["items"][0]["score"], 4);
    assert_eq!(json["items"][1]["kind"], "MeditationSession");
    assert_eq!(json["items"][1]["title"], "Meditation 9");
}

#[tokio::test]
async fn timeline_notes_only_drops_events_without_notes() {
    let app = app(vec![event(9, at_noon(9, 20))]);
    create_entry(&app, member_actor(), 4, Some("note")).await;
    create_entry(&app, member_actor(), 3, None).await;

    let (_, json) = send(&app, call("GET", "/timeline?notes_only=true")).await;

    assert_eq!(json["items"].as_array().unwrap().len(), 1);
    assert_eq!(json["items"][0]["note"], "note");
}
