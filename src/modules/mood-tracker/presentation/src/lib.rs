use mood_tracker_application::commands::create_mood_entry::CreateMoodEntryCommand;
use mood_tracker_application::commands::delete_mood_entry::DeleteMoodEntryCommand;
use mood_tracker_application::commands::update_mood_entry::UpdateMoodEntryCommand;
use mood_tracker_application::ports::{CommandHandler, Page, QueryHandler};
use mood_tracker_application::queries::get_mood_history::GetMoodHistoryQuery;
use mood_tracker_application::queries::get_mood_statistics::GetMoodStatisticsQuery;
use mood_tracker_application::queries::get_timeline::GetTimelineQuery;
use mood_tracker_application::queries::{
    get_mood_statistics::MoodStatistics, get_timeline::TimelineEvent,
};
use mood_tracker_domain::MoodEntry;
use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use kernel::Actor;

mod response;
mod router;

pub use response::{ensure_browser_origin, ApiError};
pub use router::mood_tracker_router;

pub struct MoodTrackerDependencies {
    pub create: Arc<dyn CommandHandler<CreateMoodEntryCommand, Output = MoodEntry>>,
    pub update: Arc<dyn CommandHandler<UpdateMoodEntryCommand, Output = MoodEntry>>,
    pub delete: Arc<dyn CommandHandler<DeleteMoodEntryCommand, Output = ()>>,
    pub get_mood_history: Arc<dyn QueryHandler<GetMoodHistoryQuery, Output = Page<MoodEntry>>>,
    pub get_mood_statistics: Arc<dyn QueryHandler<GetMoodStatisticsQuery, Output = MoodStatistics>>,
    pub get_timeline: Arc<dyn QueryHandler<GetTimelineQuery, Output = Vec<TimelineEvent>>>,
    pub medioss_origin: Arc<String>,
}

pub type SharedDependencies = Arc<MoodTrackerDependencies>;

pub struct CurrentActor(pub Actor);

impl<S> FromRequestParts<S> for CurrentActor
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .map(CurrentActor)
            .ok_or_else(ApiError::unauthorized)
    }
}
