use std::sync::Arc;

use super::get_mood_history_query::GetMoodHistoryQuery;
use super::get_mood_history_query_validator::GetMoodHistoryQueryValidator;
use crate::models::MoodRange;
use crate::ports::{Clock, MoodEntryRepository, Page, QueryHandler};
use crate::ApplicationError;
use chrono::{DateTime, FixedOffset, Utc};
use mood_tracker_domain::MoodEntry;

pub struct GetMoodHistoryQueryHandler {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
    validator: GetMoodHistoryQueryValidator,
}

impl GetMoodHistoryQueryHandler {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            entries,
            clock,
            validator: GetMoodHistoryQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetMoodHistoryQuery> for GetMoodHistoryQueryHandler {
    type Output = Page<MoodEntry>;

    async fn handle(
        &self,
        query: GetMoodHistoryQuery,
    ) -> Result<Page<MoodEntry>, ApplicationError> {
        self.validator.validate(&query)?;
        let offset = self.validator.local_offset(&query)?;
        get_mood_history(
            self.entries.as_ref(),
            query.user_id,
            query.range,
            offset,
            self.clock.now().await,
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
    offset: FixedOffset,
    now: DateTime<Utc>,
    limit: u16,
    cursor: Option<(DateTime<Utc>, i64)>,
) -> Result<Page<MoodEntry>, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
{
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

#[cfg(test)]
#[path = "get_mood_history_query_handler.tests.rs"]
mod tests;
