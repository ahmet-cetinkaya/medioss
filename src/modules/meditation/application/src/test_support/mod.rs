//! Test fakes for the meditation application ports (identity test_support
//! style: plain state behind `Arc<Mutex<..>>`, one fake per port).

mod fake_blob_store;
mod fake_category_repository;
mod fake_converter;
mod fake_cover_validator;
mod fake_favorite_repository;
mod fake_meditation_repository;
mod fake_membership_repository;
mod fake_note_repository;
mod fake_playback_repository;
mod fake_program_repository;
mod fake_session_repository;
mod fake_upload_staging;
mod fake_validator;
mod fixed_clock;
mod repo_state;
mod test_deps;
mod test_limits;
mod user_ids;

pub use fake_blob_store::FakeBlobStore;
pub use fake_category_repository::FakeCategoryRepository;
pub use fake_converter::FakeConverter;
pub use fake_cover_validator::{FailingCoverValidator, FakeCoverValidator};
pub use fake_favorite_repository::FakeFavoriteRepository;
pub use fake_meditation_repository::FakeMeditationRepository;
pub use fake_membership_repository::FakeMembershipRepository;
pub use fake_note_repository::FakeNoteRepository;
pub use fake_playback_repository::FakePlaybackRepository;
pub use fake_program_repository::FakeProgramRepository;
pub use fake_session_repository::FakeSessionRepository;
pub use fake_upload_staging::FakeUploadStaging;
pub use fake_validator::{sniff_audio, FakeValidator};
pub use fixed_clock::FixedClock;
pub use repo_state::RepoState;
pub use test_deps::TestDeps;
pub use test_limits::{TEST_MAX_COVER_BYTES, TEST_MAX_UPLOAD_BYTES};
pub use user_ids::{user_a, user_b};
