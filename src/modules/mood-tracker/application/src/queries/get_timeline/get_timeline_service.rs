use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use kernel::UserId;

use super::get_timeline::GetTimeline;
use super::get_timeline_handler::GetTimelineHandler;
use crate::queries::get_mood_statistics::MoodRange;
use super::timeline_event::{TimelineEvent, TimelineKind};
use crate::ports::{
    MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
    MoodEntryRepository,
};
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

pub struct GetTimelineService {
    entries: Arc<dyn MoodEntryRepository>,
    meditation: Arc<dyn MeditationTimelineSource>,
}

impl GetTimelineService {
    pub fn new(
        entries: Arc<dyn MoodEntryRepository>,
        meditation: Arc<dyn MeditationTimelineSource>,
    ) -> Self {
        Self { entries, meditation }
    }
}

#[async_trait::async_trait]
impl GetTimelineHandler for GetTimelineService {
    async fn handle(&self, query: GetTimeline) -> Result<Vec<TimelineEvent>, ApplicationError> {
        get_timeline(
            self.entries.as_ref(),
            self.meditation.as_ref(),
            query.user_id,
            query.range,
            query.tz_offset_minutes,
            query.now,
            query.limit,
            query.notes_only,
        )
        .await
    }
}

pub(crate) async fn get_timeline<M, S>(
    moods: &M,
    meditation: &S,
    user_id: UserId,
    range: MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
    limit: u16,
    notes_only: bool,
) -> Result<Vec<TimelineEvent>, ApplicationError>
where
    M: MoodEntryRepository + ?Sized,
    S: MeditationTimelineSource + ?Sized,
{
    if !(1..=100).contains(&limit) {
        return Err(ApplicationError::InvalidInput(
            "page size must be between 1 and 100",
        ));
    }
    let offset = FixedOffset::east_opt(i32::from(tz_offset_minutes) * 60)
        .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
    let today = now.with_timezone(&offset).date_naive();
    let start = range.start(today);
    let end = range.end(today);
    let includes = |at: DateTime<Utc>| {
        let day = at.with_timezone(&offset).date_naive();
        day <= end && start.is_none_or(|start| day >= start)
    };
    let mut events = moods
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
        .into_iter()
        .filter(|entry| includes(entry.recorded_at()) && (!notes_only || entry.note().is_some()))
        .map(mood_event)
        .collect::<Vec<_>>();
    events.extend(
        meditation
            .list_for_user(user_id)
            .await
            .map_err(ApplicationError::Repository)?
            .into_iter()
            .filter(|entry| includes(entry.recorded_at) && (!notes_only || entry.note.is_some()))
            .map(meditation_event),
    );
    events.sort_by(|left, right| {
        right
            .recorded_at
            .cmp(&left.recorded_at)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(events)
}

fn mood_event(entry: MoodEntry) -> TimelineEvent {
    TimelineEvent {
        id: entry.id(),
        kind: TimelineKind::Mood,
        title: "Mood entry".to_owned(),
        score: Some(entry.score().value()),
        note: entry.note().map(ToOwned::to_owned),
        recorded_at: entry.recorded_at(),
    }
}

fn meditation_event(entry: MeditationTimelineEvent) -> TimelineEvent {
    TimelineEvent {
        id: entry.id,
        kind: match entry.kind {
            MeditationTimelineEventKind::Note => TimelineKind::MeditationNote,
            MeditationTimelineEventKind::CompletedSession => TimelineKind::MeditationSession,
        },
        title: entry.meditation_name,
        score: None,
        note: entry.note,
        recorded_at: entry.recorded_at,
    }
}
