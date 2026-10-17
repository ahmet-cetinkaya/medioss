//! Meditation presentation: Axum routes, extractors, DTOs.

mod audio_stream;
mod audio_upload;
mod byte_range;
mod categories;
mod handlers_notes;
mod handlers_playback;
mod handlers_statistics;
mod meditation_response;
mod meditations;
mod program_activity;
mod program_cover;
mod program_members;
mod programs;

pub mod extractors;
pub mod response;
pub mod router;

pub use extractors::{AdminActor, CurrentActor};
pub use response::{application_error_response, ensure_browser_origin, ApiError};
pub use router::{meditation_router, MeditationDependencies, SharedDependencies};

#[cfg(test)]
#[path = "router_tests/mod.rs"]
mod router_tests;
