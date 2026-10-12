use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use kernel::{Actor, Role, UserId};
use mood_tracker_application::{
    ports::{MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource},
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
    let deps = MoodTrackerDependencies {
        entries: Arc::new(FakeMoodEntryRepository::default()),
        meditation: Arc::new(FakeMeditationTimelineSource(events)),
        clock: Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0)
                .single()
                .expect("test instant"),
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
