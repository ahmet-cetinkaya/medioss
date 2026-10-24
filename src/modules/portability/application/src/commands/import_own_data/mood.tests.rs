use super::*;
use crate::test_support::fixtures::{entry, user};
use crate::test_support::{FakeMoodStore, MoodSinkFailingOnSecondInsert};

#[tokio::test]
async fn existing_keys_collects_time_score_and_note() {
    let store = FakeMoodStore::default();
    let first = entry(4, Some("good"), 10);
    let second = entry(2, None, 20);
    store
        .entries
        .lock()
        .unwrap()
        .extend([first.clone(), second.clone()]);

    let keys = existing_mood_keys(user(), &store).await.unwrap();

    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&(first.recorded_at, 4, Some("good".into()))));
    assert!(keys.contains(&(second.recorded_at, 2, None)));
}

#[tokio::test]
async fn existing_keys_of_empty_store_is_empty() {
    let keys = existing_mood_keys(user(), &FakeMoodStore::default())
        .await
        .unwrap();
    assert!(keys.is_empty());
}

#[tokio::test]
async fn apply_inserts_new_entries_and_counts_them() {
    let sink = FakeMoodStore::default();
    let mut report = ImportReport::default();

    apply_mood_entries(
        user(),
        vec![entry(3, None, 5), entry(4, Some("x"), 6)],
        HashSet::new(),
        &sink,
        &mut report,
    )
    .await
    .unwrap();

    assert_eq!(report.mood_created, 2);
    assert_eq!(report.mood_duplicates_skipped, 0);
    assert_eq!(sink.entries.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn apply_skips_entries_matching_existing_keys() {
    let sink = FakeMoodStore::default();
    let known = entry(3, Some("same"), 5);
    let existing = HashSet::from([(known.recorded_at, 3, Some("same".to_string()))]);
    let mut report = ImportReport::default();

    apply_mood_entries(user(), vec![known], existing, &sink, &mut report)
        .await
        .unwrap();

    assert_eq!(report.mood_duplicates_skipped, 1);
    assert_eq!(report.mood_created, 0);
    assert!(sink.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn apply_treats_different_score_or_note_as_distinct() {
    let sink = FakeMoodStore::default();
    let base = entry(3, Some("a"), 5);
    let mut other_score = base.clone();
    other_score.score = 4;
    let mut other_note = base.clone();
    other_note.note = None;
    let existing = HashSet::from([(base.recorded_at, 3, Some("a".to_string()))]);
    let mut report = ImportReport::default();

    apply_mood_entries(
        user(),
        vec![other_score, other_note],
        existing,
        &sink,
        &mut report,
    )
    .await
    .unwrap();

    assert_eq!(report.mood_created, 2);
}

#[tokio::test]
async fn apply_skips_repeat_within_the_same_batch() {
    let sink = FakeMoodStore::default();
    let repeated = entry(3, None, 5);
    let mut report = ImportReport::default();

    apply_mood_entries(
        user(),
        vec![repeated.clone(), repeated],
        HashSet::new(),
        &sink,
        &mut report,
    )
    .await
    .unwrap();

    assert_eq!(report.mood_created, 1);
    assert_eq!(report.mood_duplicates_skipped, 1);
    assert_eq!(sink.entries.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn apply_stops_at_first_sink_error_keeping_earlier_counts() {
    let sink = MoodSinkFailingOnSecondInsert {
        inserted: Default::default(),
    };
    let mut report = ImportReport::default();

    let outcome = apply_mood_entries(
        user(),
        vec![entry(1, None, 1), entry(2, None, 2), entry(3, None, 3)],
        HashSet::new(),
        &sink,
        &mut report,
    )
    .await;

    assert_eq!(outcome, Err("disk full".to_string()));
    assert_eq!(report.mood_created, 1);
    assert_eq!(sink.inserted.lock().unwrap().len(), 1);
}
