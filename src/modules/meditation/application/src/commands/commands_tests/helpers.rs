use chrono::Utc;
use kernel::UserId;

use super::super::*;
use crate::test_support::{
    FakeBlobStore, FakeCategoryRepository, FakeConverter, FakeCoverValidator,
    FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository, FakePlaybackRepository,
    FakeProgramRepository, FakeSessionRepository, FakeValidator, FixedClock, RepoState,
};

pub(super) struct Deps {
    pub(super) meditations: FakeMeditationRepository,
    pub(super) programs: FakeProgramRepository,
    pub(super) categories: FakeCategoryRepository,
    pub(super) memberships: FakeMembershipRepository,
    pub(super) sessions: FakeSessionRepository,
    pub(super) notes: FakeNoteRepository,
    pub(super) playback: FakePlaybackRepository,
    pub(super) store: FakeBlobStore,
    pub(super) clock: FixedClock,
}

impl Deps {
    pub(super) fn new() -> Self {
        Self {
            meditations: FakeMeditationRepository(RepoState::new()),
            programs: FakeProgramRepository(RepoState::new()),
            categories: FakeCategoryRepository(RepoState::new()),
            memberships: FakeMembershipRepository(RepoState::new()),
            sessions: FakeSessionRepository(RepoState::new()),
            notes: FakeNoteRepository(RepoState::new()),
            playback: FakePlaybackRepository(RepoState::new()),
            store: FakeBlobStore::new(),
            clock: FixedClock(Utc::now()),
        }
    }

    /// A ready program whose folder receives uploads in tests.
    pub(super) async fn seed_program_id(&self) -> i64 {
        create_program(
            &self.programs,
            &FakeCoverValidator,
            &self.store,
            &self.clock,
            "Holder Program",
            None,
            None,
        )
        .await
        .expect("holder program")
        .id
    }
    /// A ready-made stored meditation row the other commands can reference.
    pub(super) async fn seeded_meditation(&self) -> meditation_domain::Meditation {
        let program_id = self.seed_program_id().await;
        upload_meditation(
            &self.meditations,
            &self.memberships,
            &self.programs,
            &FakeValidator,
            &FakeConverter { fail: false },
            &self.store,
            &self.clock,
            "Seed",
            program_id,
            b"OggS....OpusHead".to_vec(),
            None,
        )
        .await
        .expect("seed upload")
    }
}

pub(super) fn user_a() -> UserId {
    UserId::new(1)
}

pub(super) fn user_b() -> UserId {
    UserId::new(2)
}
