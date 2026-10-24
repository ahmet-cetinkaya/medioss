use std::sync::Arc;

use super::get_timeline::GetTimeline;
use super::get_timeline_handler::GetTimelineHandler;
use super::timeline_event::{TimelineEvent, TimelineKind};
use crate::ports::{
    Clock, MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
    MoodEntryRepository,
};
use crate::ApplicationError;
use chrono::{DateTime, FixedOffset, Utc};
use mood_tracker_domain::MoodEntry;

pub struct GetTimelineService {
    entries: Arc<dyn MoodEntryRepository>,
    meditation: Arc<dyn MeditationTimelineSource>,
    clock: Arc<dyn Clock>,
}

impl GetTimelineService {
    pub fn new(
        entries: Arc<dyn MoodEntryRepository>,
        meditation: Arc<dyn MeditationTimelineSource>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            entries,
            meditation,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl GetTimelineHandler for GetTimelineService {
    async fn handle(&self, query: GetTimeline) -> Result<Vec<TimelineEvent>, ApplicationError> {
        let now = self.clock.now().await;
        if !(1..=100).contains(&query.limit) {
            return Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100",
            ));
        }
        let offset = FixedOffset::east_opt(i32::from(query.tz_offset_minutes) * 60)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
        let today = now.with_timezone(&offset).date_naive();
        let start = query.range.start(today);
        let end = query.range.end(today);
        let includes = |at: DateTime<Utc>| {
            let day = at.with_timezone(&offset).date_naive();
            day <= end && start.is_none_or(|start| day >= start)
        };
        let mut events = self
            .entries
            .list_all_for_user(query.user_id)
            .await
            .map_err(ApplicationError::Repository)?
            .into_iter()
            .filter(|entry| {
                includes(entry.recorded_at()) && (!query.notes_only || entry.note().is_some())
            })
            .map(mood_event)
            .collect::<Vec<_>>();
        events.extend(
            self.meditation
                .list_for_user(query.user_id)
                .await
                .map_err(ApplicationError::Repository)?
                .into_iter()
                .filter(|entry| {
                    includes(entry.recorded_at) && (!query.notes_only || entry.note.is_some())
                })
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

#[cfg(test)]
#[path = "get_timeline_service.tests.rs"]
mod tests;
