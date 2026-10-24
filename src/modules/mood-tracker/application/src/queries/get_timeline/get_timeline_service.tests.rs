use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodScore;

use super::GetTimelineService;
use crate::models::MoodRange;
use crate::ports::{
    MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
    MoodEntryRepository,
};
use crate::queries::get_timeline::{GetTimeline, GetTimelineHandler, TimelineKind};
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, FixedClock, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

const USER: i64 = 1;
const OTHER_USER: i64 = 2;

struct FakeMeditation(Result<Vec<MeditationTimelineEvent>, String>);

#[async_trait]
impl MeditationTimelineSource for FakeMeditation {
    async fn list_for_user(
        &self,
        _user_id: UserId,
    ) -> Result<Vec<MeditationTimelineEvent>, String> {
        self.0.clone()
    }
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

fn at(day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, day, hour, 0, 0).unwrap()
}

fn meditation(
    id: i64,
    kind: MeditationTimelineEventKind,
    note: Option<&str>,
    recorded_at: DateTime<Utc>,
) -> MeditationTimelineEvent {
    MeditationTimelineEvent {
        id,
        meditation_name: format!("Meditation {id}"),
        note: note.map(str::to_owned),
        recorded_at,
        kind,
    }
}

async fn seed(
    entries: &FakeMoodEntryRepository,
    user: i64,
    note: Option<&str>,
    when: DateTime<Utc>,
) {
    entries
        .insert(
            UserId::new(user),
            MoodScore::new(4).unwrap(),
            note.map(str::to_owned),
            when,
        )
        .await
        .unwrap();
}

fn query(notes_only: bool) -> GetTimeline {
    GetTimeline {
        user_id: UserId::new(USER),
        range: MoodRange::AllTime,
        tz_offset_minutes: 0,
        limit: 50,
        notes_only,
    }
}

fn service(
    entries: &Arc<FakeMoodEntryRepository>,
    events: Vec<MeditationTimelineEvent>,
) -> GetTimelineService {
    GetTimelineService::new(
        entries.clone(),
        Arc::new(FakeMeditation(Ok(events))),
        Arc::new(FixedClock(now())),
    )
}

#[tokio::test]
async fn nothing_recorded_gives_an_empty_timeline() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    assert!(service(&entries, Vec::new())
        .handle(query(false))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn merges_mood_and_meditation_events_newest_first() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, Some("felt calm"), at(27, 9)).await;
    seed(&entries, OTHER_USER, None, at(28, 9)).await;
    let events = vec![
        meditation(
            7,
            MeditationTimelineEventKind::CompletedSession,
            None,
            at(28, 8),
        ),
        meditation(
            8,
            MeditationTimelineEventKind::Note,
            Some("deep"),
            at(26, 8),
        ),
    ];

    let timeline = service(&entries, events)
        .handle(query(false))
        .await
        .unwrap();

    let shape: Vec<_> = timeline
        .iter()
        .map(|event| (event.kind, event.recorded_at))
        .collect();
    assert_eq!(
        shape,
        [
            (TimelineKind::MeditationSession, at(28, 8)),
            (TimelineKind::Mood, at(27, 9)),
            (TimelineKind::MeditationNote, at(26, 8)),
        ]
    );
}

#[tokio::test]
async fn mood_events_carry_score_and_note_and_meditation_events_carry_their_name() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, Some("felt calm"), at(27, 9)).await;
    let events = vec![meditation(
        7,
        MeditationTimelineEventKind::CompletedSession,
        None,
        at(28, 8),
    )];

    let timeline = service(&entries, events)
        .handle(query(false))
        .await
        .unwrap();

    assert_eq!(timeline[0].title, "Meditation 7");
    assert_eq!(timeline[0].score, None);
    assert_eq!(timeline[1].title, "Mood entry");
    assert_eq!(timeline[1].score, Some(4));
    assert_eq!(timeline[1].note.as_deref(), Some("felt calm"));
}

#[tokio::test]
async fn ties_on_time_put_mood_before_meditation_kinds() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, None, at(27, 9)).await;
    let events = vec![meditation(
        9,
        MeditationTimelineEventKind::CompletedSession,
        None,
        at(27, 9),
    )];

    let timeline = service(&entries, events)
        .handle(query(false))
        .await
        .unwrap();

    let kinds: Vec<_> = timeline.iter().map(|event| event.kind).collect();
    assert_eq!(kinds, [TimelineKind::Mood, TimelineKind::MeditationSession]);
}

#[tokio::test]
async fn notes_only_drops_events_without_notes() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, Some("kept"), at(27, 9)).await;
    seed(&entries, USER, None, at(27, 10)).await;
    let events = vec![
        meditation(
            1,
            MeditationTimelineEventKind::CompletedSession,
            None,
            at(28, 8),
        ),
        meditation(
            2,
            MeditationTimelineEventKind::Note,
            Some("kept too"),
            at(26, 8),
        ),
    ];

    let timeline = service(&entries, events).handle(query(true)).await.unwrap();

    let notes: Vec<_> = timeline
        .iter()
        .map(|event| event.note.clone().unwrap())
        .collect();
    assert_eq!(notes, ["kept", "kept too"]);
}

#[tokio::test]
async fn range_filters_both_sources() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, None, at(1, 9)).await;
    seed(&entries, USER, None, at(28, 9)).await;
    let events = vec![
        meditation(1, MeditationTimelineEventKind::Note, None, at(2, 8)),
        meditation(2, MeditationTimelineEventKind::Note, None, at(28, 10)),
    ];
    let mut recent = query(false);
    recent.range = MoodRange::Last7Days;

    let timeline = service(&entries, events).handle(recent).await.unwrap();

    assert_eq!(timeline.len(), 2);
    assert!(timeline.iter().all(|event| event.recorded_at >= at(22, 0)));
}

#[tokio::test]
async fn page_size_outside_one_to_one_hundred_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    for limit in [0, 101] {
        let mut bad = query(false);
        bad.limit = limit;
        assert_eq!(
            service(&entries, Vec::new()).handle(bad).await,
            Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100"
            )),
            "limit {limit}"
        );
    }
}

#[tokio::test]
async fn impossible_timezone_offset_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let mut bad = query(false);
    bad.tz_offset_minutes = 1440;

    assert_eq!(
        service(&entries, Vec::new()).handle(bad).await,
        Err(ApplicationError::InvalidInput("invalid timezone offset"))
    );
}

#[tokio::test]
async fn mood_repository_failure_is_reported_as_a_repository_error() {
    let service = GetTimelineService::new(
        Arc::new(FailingMoodEntryRepository),
        Arc::new(FakeMeditation(Ok(Vec::new()))),
        Arc::new(FixedClock(now())),
    );

    assert_eq!(
        service.handle(query(false)).await,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}

#[tokio::test]
async fn meditation_source_failure_is_reported_as_a_repository_error() {
    let service = GetTimelineService::new(
        Arc::new(FakeMoodEntryRepository::default()),
        Arc::new(FakeMeditation(Err("meditation offline".to_owned()))),
        Arc::new(FixedClock(now())),
    );

    assert_eq!(
        service.handle(query(false)).await,
        Err(ApplicationError::Repository(
            "meditation offline".to_owned()
        ))
    );
}
