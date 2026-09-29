//! Meditation presentation: Axum routes, extractors, DTOs.

// Handlers live beside the router; #[path] keeps router.rs a single module.
#[path = "handlers_audio.rs"]
pub mod handlers_audio;
#[path = "handlers_notes.rs"]
pub mod handlers_notes;
#[path = "handlers_playback.rs"]
pub mod handlers_playback;
#[path = "handlers_programs.rs"]
pub mod handlers_programs;
#[path = "handlers_statistics.rs"]
pub mod handlers_statistics;

pub mod extractors;
pub mod response;
pub mod router;

pub use extractors::{AdminActor, CurrentActor};
pub use response::{application_error_response, ensure_browser_origin, ApiError};
pub use router::{meditation_router, MeditationDependencies, SharedDependencies};

#[cfg(test)]
#[path = "router.tests.rs"]
mod router_tests;
