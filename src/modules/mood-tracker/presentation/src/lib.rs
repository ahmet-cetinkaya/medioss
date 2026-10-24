use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use kernel::Actor;
use mood_tracker_application::commands::create_mood_entry::CreateMoodEntryHandler;
use mood_tracker_application::commands::delete_mood_entry::DeleteMoodEntryHandler;
use mood_tracker_application::commands::update_mood_entry::UpdateMoodEntryHandler;
use mood_tracker_application::ports::Clock;
use mood_tracker_application::queries::get_mood_history::MoodHistoryHandler;
use mood_tracker_application::queries::get_mood_statistics::GetMoodStatisticsHandler;
use mood_tracker_application::queries::get_timeline::GetTimelineHandler;

mod response;
mod router;

pub use response::{ensure_browser_origin, ApiError};
pub use router::mood_tracker_router;

#[cfg(test)]
#[path = "router.tests.rs"]
mod router_tests;

pub struct MoodTrackerDependencies {
    pub create: Arc<dyn CreateMoodEntryHandler>,
    pub update: Arc<dyn UpdateMoodEntryHandler>,
    pub delete: Arc<dyn DeleteMoodEntryHandler>,
    pub list_mood_entries: Arc<dyn MoodHistoryHandler>,
    pub mood_statistics: Arc<dyn GetMoodStatisticsHandler>,
    pub timeline: Arc<dyn GetTimelineHandler>,
    pub clock_now: Arc<dyn Clock>,
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
