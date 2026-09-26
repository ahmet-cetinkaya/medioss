//! Persistence: repositories implementing the application ports over
//! SeaORM/SQLite. One file per repository (identity convention).

pub mod category_repository;
pub mod clock;
pub mod entities;
pub mod meditation_repository;
pub mod membership_repository;
pub mod note_repository;
pub mod playback_repository;
pub mod program_repository;
pub mod session_repository;

pub use category_repository::SeaOrmCategoryRepository;
pub use clock::SystemClock;
pub use meditation_repository::SeaOrmMeditationRepository;
pub use membership_repository::SeaOrmMembershipRepository;
pub use note_repository::SeaOrmNoteRepository;
pub use playback_repository::SeaOrmPlaybackRepository;
pub use program_repository::SeaOrmProgramRepository;
pub use session_repository::SeaOrmSessionRepository;

#[cfg(test)]
#[path = "persistence.tests.rs"]
mod repository_tests;
