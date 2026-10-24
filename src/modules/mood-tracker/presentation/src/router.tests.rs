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

use crate::{mood_tracker_router, MoodTrackerDependencies};

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
