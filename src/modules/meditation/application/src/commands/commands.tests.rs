use chrono::{Duration, Utc};
use kernel::UserId;

use super::*;
use crate::ports::{
    CoverNormalizationError, MeditationRepository, PlaybackStateRepository, ProgramRepository,
    SessionRepository,
};
use crate::test_support::{
    FailingCoverValidator, FakeBlobStore, FakeCategoryRepository, FakeConverter,
    FakeCoverValidator, FakeMeditationRepository, FakeMembershipRepository, FakeNoteRepository,
    FakePlaybackRepository, FakeProgramRepository, FakeSessionRepository, FakeValidator,
    FixedClock, RepoState,
};

struct Deps {
    meditations: FakeMeditationRepository,
    programs: FakeProgramRepository,
    categories: FakeCategoryRepository,
    memberships: FakeMembershipRepository,
    sessions: FakeSessionRepository,
    notes: FakeNoteRepository,
    playback: FakePlaybackRepository,
    store: FakeBlobStore,
    clock: FixedClock,
}

impl Deps {
    fn new() -> Self {
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
    async fn seed_program_id(&self) -> i64 {
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
    async fn seeded_meditation(&self) -> meditation_domain::Meditation {
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

fn user_a() -> UserId {
    UserId::new(1)
}

fn user_b() -> UserId {
    UserId::new(2)
}

#[tokio::test]
async fn upload_rejects_blank_and_oversized_names_and_garbage_bytes() {
    let deps = Deps::new();
    let converter = FakeConverter { fail: false };

    let blank = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        "   ",
        deps.seed_program_id().await,
        b"OggSx".to_vec(),
        None,
    )
    .await;
    assert!(matches!(blank, Err(ApplicationError::InvalidInput(_))));

    let long_name = "x".repeat(MAX_NAME_LENGTH + 1);
    let too_long = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        &long_name,
        deps.seed_program_id().await,
        b"OggSx".to_vec(),
        None,
    )
    .await;
    assert!(matches!(too_long, Err(ApplicationError::InvalidInput(_))));

    let garbage = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        "Valid name",
        deps.seed_program_id().await,
        b"not audio at all".to_vec(),
        None,
    )
    .await;
    assert_eq!(garbage.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upload_rejects_payloads_over_the_cap_before_anything_stored() {
    let deps = Deps::new();
    let mut oversized = vec![0u8; MAX_UPLOAD_BYTES + 1];
    oversized[0] = 0xFF;
    oversized[1] = 0xFB;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Big",
        deps.seed_program_id().await,
        oversized,
        None,
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_at_the_store_leaves_no_blob() {
    // The store itself fails at `put`, before any row exists — nothing to
    // compensate, so the only assertion is that no blob was left behind.
    let deps = Deps::new();
    *deps.store.failures.lock().unwrap() = true;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Valid",
        deps.seed_program_id().await,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await;
    assert!(matches!(result, Err(ApplicationError::Storage(_))));
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_after_store_deletes_the_orphaned_blob() {
    // The store succeeds but the membership insert fails afterward — the
    // meditation row and its blob must both be rolled back rather than left
    // as an unreachable, program-less meditation.
    let deps = Deps::new();
    *deps.memberships.0.membership_add_failures.lock().unwrap() = true;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Valid",
        deps.seed_program_id().await,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await;
    assert!(matches!(result, Err(ApplicationError::Repository(_))));
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_removes_row_bytes_memberships_and_notes() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Program",
        None,
        None,
    )
    .await
    .expect("program");
    add_member(
        &deps.meditations,
        &deps.programs,
        &deps.memberships,
        &deps.clock,
        program.id,
        meditation.id(),
    )
    .await
    .expect("member");
    create_note(
        &deps.meditations,
        &deps.notes,
        &deps.clock,
        user_a(),
        meditation.id(),
        "felt calm",
    )
    .await
    .expect("note");

    delete_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.notes,
        &deps.store,
        meditation.id(),
    )
    .await
    .expect("delete");

    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
    assert!(deps.memberships.0.memberships.lock().unwrap().is_empty());
    assert!(deps.notes.0.notes.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_unknown_meditation_is_not_found() {
    let deps = Deps::new();
    let result = delete_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.notes,
        &deps.store,
        999,
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::NotFound);
}

#[tokio::test]
async fn completion_is_idempotent_sets_completed_and_resets_position() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    let finished = started + Duration::seconds(60);

    record_position(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        30_000,
    )
    .await
    .expect("position");

    let first = record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        finished,
    )
    .await
    .expect("completion");
    let retry = record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        finished,
    )
    .await
    .expect("retry");

    assert_eq!(first.id, retry.id);
    assert_eq!(
        deps.sessions
            .count_for(user_a(), meditation.id())
            .await
            .unwrap(),
        1
    );
    let state = deps
        .playback
        .get(user_a(), meditation.id())
        .await
        .unwrap()
        .expect("state");
    assert_eq!(state.position_ms, 0);
    assert!(state.completed);
}

#[tokio::test]
async fn completion_state_is_per_user() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        started + Duration::seconds(30),
    )
    .await
    .expect("a completes");

    let b_state = deps.playback.get(user_b(), meditation.id()).await.unwrap();
    assert!(b_state.is_none());
}

#[tokio::test]
async fn toggle_completed_never_touches_sessions_or_position() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        started + Duration::seconds(30),
    )
    .await
    .expect("completion");

    set_completed(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        false,
    )
    .await
    .expect("toggle off");
    assert_eq!(
        deps.sessions
            .count_for(user_a(), meditation.id())
            .await
            .unwrap(),
        1
    );
    let state = deps
        .playback
        .get(user_a(), meditation.id())
        .await
        .unwrap()
        .unwrap();
    assert!(!state.completed);
}

#[tokio::test]
async fn category_names_must_be_unique_and_nonempty() {
    let deps = Deps::new();
    create_category(&deps.categories, &deps.clock, "Morning")
        .await
        .expect("first");
    let duplicate = create_category(&deps.categories, &deps.clock, "Morning").await;
    assert_eq!(duplicate.unwrap_err(), ApplicationError::DuplicateCategory);
    let empty = create_category(&deps.categories, &deps.clock, "   ").await;
    assert!(matches!(empty, Err(ApplicationError::InvalidInput(_))));
}

#[tokio::test]
async fn renaming_to_a_name_inserted_after_the_duplicate_check_is_a_duplicate() {
    let deps = Deps::new();
    let renamed = create_category(&deps.categories, &deps.clock, "Morning")
        .await
        .expect("renamed category");
    create_category(&deps.categories, &deps.clock, "Evening")
        .await
        .expect("existing category");

    let error = rename_category(&deps.categories, &deps.clock, renamed.id, "Evening")
        .await
        .expect_err("repository duplicate maps to application error");

    assert_eq!(error, ApplicationError::DuplicateCategory);
}

#[tokio::test]
async fn deleting_a_category_leaves_its_programs_uncategorized() {
    let deps = Deps::new();
    let category = create_category(&deps.categories, &deps.clock, "Evening")
        .await
        .expect("category");
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Wind-down",
        None,
        None,
    )
    .await
    .expect("program");
    set_program_category(
        &deps.programs,
        &deps.categories,
        program.id,
        Some(category.id),
    )
    .await
    .expect("assign");

    delete_category(&deps.categories, &deps.programs, category.id)
        .await
        .expect("delete category");

    let stored = deps
        .programs
        .find_by_id(program.id)
        .await
        .unwrap()
        .expect("program survives");
    assert_eq!(stored.category_id, None);
}

#[tokio::test]
async fn deleting_a_program_keeps_meditations_and_cleans_cover_and_members() {
    let deps = Deps::new();
    let png = vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Covered",
        Some("desc"),
        Some(png),
    )
    .await
    .expect("program with cover");
    // Uploaded directly into the program being deleted, so its file actually
    // lives in that program's folder — exercising the relocation branch,
    // not the "stayed in a different program's folder" no-op path.
    let meditation = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Home",
        program.id,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await
    .expect("home upload");
    let original_key = meditation.storage_key().to_owned();
    assert!(original_key.starts_with(&format!("{}/", program.folder)));

    delete_program(
        &deps.programs,
        &deps.memberships,
        &deps.meditations,
        &deps.store,
        program.id,
    )
    .await
    .expect("delete program");

    let relocated_key = {
        let survivors = deps.meditations.0.meditations.lock().unwrap();
        let relocated = survivors
            .iter()
            .find(|row| row.id() == meditation.id())
            .expect("meditation row survives");
        assert_ne!(
            relocated.storage_key(),
            original_key,
            "file must be relocated"
        );
        assert!(relocated.storage_key().starts_with("_unfiled/"));
        relocated.storage_key().to_owned()
    };
    let memberships = deps.memberships.0.memberships.lock().unwrap();
    assert!(
        memberships.iter().all(|m| m.program_id != program.id),
        "no membership may reference the deleted program"
    );
    drop(memberships);
    // The cover blob is gone; the relocated audio blob remains (moved, not
    // duplicated).
    assert_eq!(deps.store.len(), 1);
    assert!(deps.store.contains_key(&relocated_key));
}

#[tokio::test]
async fn covers_are_validated_and_capped() {
    let deps = Deps::new();
    let garbage = b"definitely not an image".to_vec();
    let result = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Bad cover",
        None,
        Some(garbage),
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());

    let mut oversized = vec![0u8; MAX_COVER_BYTES + 12];
    oversized[0] = 0x89;
    oversized[1] = b'P';
    oversized[2] = b'N';
    oversized[3] = b'G';
    let result = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Big cover",
        None,
        Some(oversized),
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
}

#[tokio::test]
async fn cover_normalization_failures_keep_their_operational_meaning() {
    // Invalid covers stay a client error; worker failure and busy are
    // server-side conditions, and oversize output keeps 413 semantics.
    let cases = [
        (
            CoverNormalizationError::InvalidPayload,
            ApplicationError::UnsupportedMedia,
        ),
        (
            CoverNormalizationError::OutputTooLarge,
            ApplicationError::PayloadTooLarge,
        ),
        (
            CoverNormalizationError::WorkerFailed,
            ApplicationError::Storage("cover processing failed".to_owned()),
        ),
        (CoverNormalizationError::Busy, ApplicationError::Busy),
    ];
    for (error, expected) in cases {
        let deps = Deps::new();
        let result = create_program(
            &deps.programs,
            &FailingCoverValidator(error),
            &deps.store,
            &deps.clock,
            "Cover mapping",
            None,
            Some(b"RIFF....WEBP....".to_vec()),
        )
        .await;
        assert_eq!(result.unwrap_err(), expected, "{error:?}");
        assert!(deps.store.is_empty());
        assert!(deps.programs.0.programs.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn notes_are_author_scoped_and_newest_first() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    create_note(
        &deps.meditations,
        &deps.notes,
        &FixedClock(Utc::now() - Duration::seconds(60)),
        user_a(),
        meditation.id(),
        "older",
    )
    .await
    .expect("first note");
    create_note(
        &deps.meditations,
        &deps.notes,
        &deps.clock,
        user_a(),
        meditation.id(),
        "newer",
    )
    .await
    .expect("second note");

    let listed =
        crate::queries::list_notes(&deps.meditations, &deps.notes, user_a(), meditation.id())
            .await
            .expect("list");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].text, "newer");

    let b_listed =
        crate::queries::list_notes(&deps.meditations, &deps.notes, user_b(), meditation.id())
            .await
            .expect("b list");
    assert!(b_listed.is_empty());

    // B cannot update or delete A's note: indistinguishable from unknown.
    let a_note_id = listed[0].id;
    let updated = update_note(
        &deps.notes,
        &deps.clock,
        a_note_id,
        meditation.id(),
        user_b(),
        "hijack",
    )
    .await;
    assert_eq!(updated.unwrap_err(), ApplicationError::NotFound);
    let deleted = delete_note(&deps.notes, a_note_id, meditation.id(), user_b()).await;
    assert_eq!(deleted.unwrap_err(), ApplicationError::NotFound);

    // The author's own note, but addressed via the wrong meditation id, is
    // also NotFound: the nested route's meditation segment is enforced, not
    // decorative.
    let other_meditation = deps.seeded_meditation().await;
    let wrong_meditation = update_note(
        &deps.notes,
        &deps.clock,
        a_note_id,
        other_meditation.id(),
        user_a(),
        "still hijack",
    )
    .await;
    assert_eq!(wrong_meditation.unwrap_err(), ApplicationError::NotFound);
}

#[tokio::test]
async fn position_reports_are_validated_and_default_to_zero() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let unknown = record_position(&deps.meditations, &deps.playback, user_a(), 999, 1_000).await;
    assert_eq!(unknown.unwrap_err(), ApplicationError::NotFound);

    let negative = record_position(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        -1,
    )
    .await;
    assert!(matches!(negative, Err(ApplicationError::InvalidInput(_))));

    let fresh = crate::queries::get_playback_state(
        &deps.meditations,
        &deps.playback,
        user_b(),
        meditation.id(),
    )
    .await
    .expect("state");
    assert_eq!(fresh.position_ms, 0);
    assert!(!fresh.completed);
}

#[tokio::test]
async fn description_is_stored_on_upload_and_updatable() {
    let deps = Deps::new();
    let program_id = deps.seed_program_id().await;
    let uploaded = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Described",
        program_id,
        b"OggS....OpusHead".to_vec(),
        Some(" hello "),
    )
    .await
    .unwrap();
    assert_eq!(uploaded.description(), Some("hello"));

    let updated = update_meditation_description(&deps.meditations, uploaded.id(), Some("new"))
        .await
        .unwrap();
    assert_eq!(updated.description(), Some("new"));
    let cleared = update_meditation_description(&deps.meditations, uploaded.id(), Some(""))
        .await
        .unwrap();
    assert_eq!(cleared.description(), None);
    let stored = deps
        .meditations
        .find_by_id(uploaded.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.description(), None);
}

#[tokio::test]
async fn description_update_rejects_overlong_text_and_unknown_ids() {
    let deps = Deps::new();
    let seeded = deps.seeded_meditation().await;
    let long = "a".repeat(2001);
    assert_eq!(
        update_meditation_description(&deps.meditations, seeded.id(), Some(&long)).await,
        Err(ApplicationError::InvalidDescription)
    );
    assert_eq!(
        update_meditation_description(&deps.meditations, 999, Some("x")).await,
        Err(ApplicationError::NotFound)
    );
}
