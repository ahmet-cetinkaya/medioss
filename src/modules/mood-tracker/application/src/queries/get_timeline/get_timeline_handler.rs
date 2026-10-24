use super::get_timeline::GetTimeline;
use super::timeline_event::TimelineEvent;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetTimelineHandler: Send + Sync {
    async fn handle(&self, query: GetTimeline) -> Result<Vec<TimelineEvent>, ApplicationError>;
}
