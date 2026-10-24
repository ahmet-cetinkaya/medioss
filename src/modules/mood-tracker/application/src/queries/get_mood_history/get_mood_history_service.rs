use std::sync::Arc;

use super::get_mood_history::MoodHistory;
use super::get_mood_history_handler::MoodHistoryHandler;
use crate::ports::{MoodEntryRepository, Page};
use crate::queries::get_mood_statistics::MoodRange;
use crate::ApplicationError;
use chrono::{DateTime, FixedOffset, Utc};
use mood_tracker_domain::MoodEntry;

pub struct MoodHistoryService {
    entries: Arc<dyn MoodEntryRepository>,
}

impl MoodHistoryService {
    pub fn new(entries: Arc<dyn MoodEntryRepository>) -> Self {
        Self { entries }
    }
}

#[async_trait::async_trait]
impl MoodHistoryHandler for MoodHistoryService {
    async fn handle(&self, query: MoodHistory) -> Result<Page<MoodEntry>, ApplicationError> {
        get_mood_history(
            self.entries.as_ref(),
            query.user_id,
            query.range,
            query.tz_offset_minutes,
            query.now,
            query.limit,
            query.cursor,
        )
        .await
    }
}

pub(crate) async fn get_mood_history<R>(
    entries: &R,
    user_id: kernel::UserId,
    range: MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
    limit: u16,
    cursor: Option<(DateTime<Utc>, i64)>,
) -> Result<Page<MoodEntry>, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
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
    let mut items: Vec<_> = entries
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
        .into_iter()
        .filter(|entry| {
            let day = entry.recorded_at().with_timezone(&offset).date_naive();
            day <= end && start.is_none_or(|start| day >= start)
        })
        .collect();
    if let Some((at, id)) = cursor {
        items.retain(|entry| (entry.recorded_at(), entry.id()) < (at, id));
    }
    let has_more = items.len() > usize::from(limit);
    items.truncate(usize::from(limit));
    let next_cursor = has_more.then(|| {
        let last = items.last().expect("continuation implies a final item");
        (last.recorded_at(), last.id())
    });
    Ok(Page { items, next_cursor })
}
