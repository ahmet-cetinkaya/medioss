use chrono::{DateTime, Duration, FixedOffset, Utc};
use kernel::UserId;

use crate::{
    ports::{MeditationTimelineEventKind, MeditationTimelineSource, MoodEntryRepository},
    ApplicationError,
};

use super::MoodRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimelineKind {
    Mood,
    MeditationNote,
    MeditationSession,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineEvent {
    pub id: i64,
    pub kind: TimelineKind,
    pub title: String,
    pub score: Option<u8>,
    pub note: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

pub async fn get_timeline<M, S>(
    moods: &M,
    meditation: &S,
    user_id: UserId,
    range: MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
    limit: u16,
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
    let start = match range {
        MoodRange::Today => Some(today),
        MoodRange::Last7Days => Some(today - Duration::days(6)),
        MoodRange::Last30Days => Some(today - Duration::days(29)),
        MoodRange::AllTime => None,
    };
    let includes = |at: DateTime<Utc>| {
        let day = at.with_timezone(&offset).date_naive();
        day <= today && start.is_none_or(|start| day >= start)
    };
    let mut events = moods
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
        .into_iter()
        .filter(|entry| includes(entry.recorded_at()))
        .map(|entry| TimelineEvent {
            id: entry.id(),
            kind: TimelineKind::Mood,
            title: "Mood entry".to_owned(),
            score: Some(entry.score().value()),
            note: entry.note().map(ToOwned::to_owned),
            recorded_at: entry.recorded_at(),
        })
        .collect::<Vec<_>>();
    events.extend(
        meditation
            .list_for_user(user_id)
            .await
            .map_err(ApplicationError::Repository)?
            .into_iter()
            .filter(|entry| includes(entry.recorded_at))
            .map(|entry| TimelineEvent {
                id: entry.id,
                kind: match entry.kind {
                    MeditationTimelineEventKind::Note => TimelineKind::MeditationNote,
                    MeditationTimelineEventKind::CompletedSession => {
                        TimelineKind::MeditationSession
                    }
                },
                title: entry.meditation_name,
                score: None,
                note: entry.note,
                recorded_at: entry.recorded_at,
            }),
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
