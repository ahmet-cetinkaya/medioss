use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use kernel::Actor;
use mood_tracker_application::ports::{Clock, MeditationTimelineSource, MoodEntryRepository};

mod response;
mod router;

pub use response::{ensure_browser_origin, ApiError};
pub use router::mood_tracker_router;

#[cfg(test)]
#[path = "router.tests.rs"]
mod router_tests;

pub struct MoodTrackerDependencies {
    pub entries: Arc<dyn MoodEntryRepository>,
    pub meditation: Arc<dyn MeditationTimelineSource>,
    pub clock: Arc<dyn Clock>,
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
