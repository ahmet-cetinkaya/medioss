use crate::ports::CommandHandler as _;
use std::sync::Arc;

use chrono::Utc;

use super::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository, FakePlaybackRepository,
    FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
};
use crate::commands::create_program::{CreateProgramCommand, CreateProgramCommandHandler};
use crate::commands::upload_meditation::{
    UploadMeditation, UploadMeditationHandler, UploadMeditationService,
};

/// Every fake port behind an `Arc`, so a service under test and the test's
/// own assertions share one state.
pub struct TestDeps {
    pub meditations: Arc<FakeMeditationRepository>,
    pub programs: Arc<FakeProgramRepository>,
    pub categories: Arc<FakeCategoryRepository>,
    pub memberships: Arc<FakeMembershipRepository>,
    pub sessions: Arc<FakeSessionRepository>,
    pub notes: Arc<FakeNoteRepository>,
    pub playback: Arc<FakePlaybackRepository>,
    pub store: Arc<FakeBlobStore>,
    pub clock: Arc<FixedClock>,
}

impl TestDeps {
    pub fn new() -> Self {
        Self {
            meditations: Arc::new(FakeMeditationRepository(RepoState::new())),
            programs: Arc::new(FakeProgramRepository(RepoState::new())),
            categories: Arc::new(FakeCategoryRepository(RepoState::new())),
            memberships: Arc::new(FakeMembershipRepository(RepoState::new())),
            sessions: Arc::new(FakeSessionRepository(RepoState::new())),
            notes: Arc::new(FakeNoteRepository(RepoState::new())),
            playback: Arc::new(FakePlaybackRepository(RepoState::new())),
            store: Arc::new(FakeBlobStore::new()),
            clock: Arc::new(FixedClock(Utc::now())),
        }
    }

    /// A ready program whose folder receives uploads in tests.
    pub async fn seed_program(&self, title: &str) -> i64 {
        CreateProgramCommandHandler::new(
            self.programs.clone(),
            Arc::new(FakeCoverValidator),
            self.store.clone(),
            self.clock.clone(),
        )
        .handle(CreateProgramCommand {
            title: title.to_owned(),
            description: None,
            cover_bytes: None,
        })
        .await
        .expect("seed program")
        .id
    }

    /// A ready-made stored meditation row inside `program_id`.
    pub async fn seed_meditation_in(
        &self,
        program_id: i64,
        name: &str,
    ) -> meditation_domain::Meditation {
        UploadMeditationService::new(
            self.meditations.clone(),
            self.memberships.clone(),
            self.programs.clone(),
            Arc::new(FakeValidator),
            Arc::new(FakeConverter { fail: false }),
            self.store.clone(),
            self.clock.clone(),
        )
        .handle(UploadMeditation {
            name: name.to_owned(),
            program_id,
            bytes: b"OggS....OpusHead".to_vec(),
            description: None,
        })
        .await
        .expect("seed upload")
    }

    /// A ready-made stored meditation row the other use cases can reference.
    pub async fn seeded_meditation(&self) -> meditation_domain::Meditation {
        let program_id = self.seed_program("Holder Program").await;
        self.seed_meditation_in(program_id, "Seed").await
    }
}

impl Default for TestDeps {
    fn default() -> Self {
        Self::new()
    }
}
