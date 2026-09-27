use chrono::Utc;
use kernel::UserId;

use super::*;
use crate::commands::{create_program, upload_meditation};
use crate::ports::PlaybackStateRepository;
use crate::test_support::{
    FakeBlobStore, FakeConverter, FakeCoverValidator, FakeMeditationRepository,
    FakeMembershipRepository, FakePlaybackRepository, FakeProgramRepository, FakeValidator,
    FixedClock, RepoState,
};

fn user_a() -> UserId {
    UserId::new(1)
}

struct Deps {
    meditations: FakeMeditationRepository,
    programs: FakeProgramRepository,
    memberships: FakeMembershipRepository,
    playback: FakePlaybackRepository,
    store: FakeBlobStore,
    clock: FixedClock,
}

impl Deps {
    fn new() -> Self {
        let state = RepoState::new();
        Self {
            meditations: FakeMeditationRepository(state.clone()),
            programs: FakeProgramRepository(state.clone()),
            memberships: FakeMembershipRepository(state.clone()),
            playback: FakePlaybackRepository(state),
            store: FakeBlobStore::new(),
            clock: FixedClock(Utc::now()),
        }
    }

    async fn seed_program(&self, title: &str) -> i64 {
        create_program(
            &self.programs,
            &FakeCoverValidator,
            &self.store,
            &self.clock,
            title,
            None,
            None,
        )
        .await
        .expect("program")
        .id
    }

    async fn seed_meditation_in(
        &self,
        program_id: i64,
        name: &str,
    ) -> meditation_domain::Meditation {
        upload_meditation(
            &self.meditations,
            &self.memberships,
            &self.programs,
            &FakeValidator,
            &FakeConverter { fail: false },
            &self.store,
            &self.clock,
            name,
            program_id,
            b"OggS....OpusHead".to_vec(),
        )
        .await
        .expect("upload")
    }
}

#[tokio::test]
async fn continue_program_resolves_the_home_program_of_the_last_touch() {
    let deps = Deps::new();
    let program = deps.seed_program("Morning").await;
    let meditation = deps.seed_meditation_in(program, "M1").await;
    deps.playback
        .upsert_position(user_a(), meditation.id(), 1_000)
        .await
        .expect("touch");

    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.program.id, program);
    assert_eq!(found.meditation_id, meditation.id());
}

#[tokio::test]
async fn continue_program_is_none_for_an_untouched_user() {
    let deps = Deps::new();
    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query");
    assert!(found.is_none());
}

#[tokio::test]
async fn continue_program_falls_back_past_a_deleted_meditations_touch() {
    // The most recently touched meditation has no membership at all (as if
    // its program had been deleted and the meditation later removed too,
    // or the membership row otherwise vanished); the query must not give up
    // and must resolve to the next most-recently-touched live one instead.
    let deps = Deps::new();
    let program = deps.seed_program("Evening").await;
    let live = deps.seed_meditation_in(program, "Live").await;
    deps.playback
        .upsert_position(user_a(), live.id(), 500)
        .await
        .expect("touch live");
    // A touch on a meditation id with no membership row anywhere.
    let orphan_id = live.id() + 999;
    deps.playback
        .upsert_position(user_a(), orphan_id, 900)
        .await
        .expect("touch orphan");

    let found = get_continue_program(&deps.playback, &deps.memberships, &deps.programs, user_a())
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.meditation_id, live.id());
}

#[tokio::test]
async fn daily_suggestion_is_the_same_pick_for_repeated_calls_same_day() {
    let deps = Deps::new();
    let program = deps.seed_program("Library Home").await;
    deps.seed_meditation_in(program, "A").await;
    deps.seed_meditation_in(program, "B").await;
    deps.seed_meditation_in(program, "C").await;

    let first = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query")
    .expect("some");
    let second = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query")
    .expect("some");
    assert_eq!(first.meditation.id(), second.meditation.id());
    assert!(first.program.is_some());
}

#[tokio::test]
async fn daily_suggestion_is_none_for_an_empty_library() {
    let deps = Deps::new();
    let found = get_daily_suggestion(
        &deps.meditations,
        &deps.clock,
        &deps.memberships,
        &deps.programs,
    )
    .await
    .expect("query");
    assert!(found.is_none());
}

#[tokio::test]
async fn recent_programs_dedupes_by_program_most_recent_first_and_honors_limit() {
    let deps = Deps::new();
    let program_a = deps.seed_program("A").await;
    let program_b = deps.seed_program("B").await;
    let med_a1 = deps.seed_meditation_in(program_a, "A1").await;
    let med_a2 = deps.seed_meditation_in(program_a, "A2").await;
    let med_b = deps.seed_meditation_in(program_b, "B1").await;

    // Touch order (oldest to newest): A1, B1, A2. A2's touch should keep A
    // as the most-recent program (A2 is newer than B1's touch of B), and
    // A1's earlier touch of the same program must not appear twice.
    deps.playback
        .upsert_position(user_a(), med_a1.id(), 100)
        .await
        .expect("touch a1");
    deps.playback
        .upsert_position(user_a(), med_b.id(), 100)
        .await
        .expect("touch b1");
    deps.playback
        .upsert_position(user_a(), med_a2.id(), 100)
        .await
        .expect("touch a2");

    let all = list_recent_programs(
        &deps.playback,
        &deps.memberships,
        &deps.programs,
        user_a(),
        None,
    )
    .await
    .expect("query");
    assert_eq!(all.len(), 2, "each program appears once");
    assert_eq!(
        all[0].program.id, program_a,
        "most recently touched program first"
    );
    assert_eq!(all[1].program.id, program_b);

    let limited = list_recent_programs(
        &deps.playback,
        &deps.memberships,
        &deps.programs,
        user_a(),
        Some(1),
    )
    .await
    .expect("query");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].program.id, program_a);
}
