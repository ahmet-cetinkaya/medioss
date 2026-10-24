use super::super::meditation_index::meditation_index;
use super::*;
use crate::models::note_record::NoteRecord;
use crate::models::playback_record::PlaybackRecord;
use crate::models::session_record::SessionRecord;
use crate::ports::meditation_lookup::MeditationRef;
use crate::test_support::fixtures::{snapshot, ts, user};
use crate::test_support::FakePracticeSink;

const NAME: &str = "Calm";
const DURATION_MS: i64 = 1000;
const MEDITATION_ID: i64 = 5;

fn library() -> Vec<MeditationRef> {
    vec![MeditationRef {
        id: MEDITATION_ID,
        name: NAME.into(),
        duration_ms: DURATION_MS,
    }]
}

fn session(name: &str, minutes_ago: i64) -> SessionRecord {
    SessionRecord {
        meditation_name: name.into(),
        meditation_duration_ms: DURATION_MS,
        started_at: ts(minutes_ago),
        finished_at: ts(minutes_ago - 1),
    }
}

fn note(text: &str) -> NoteRecord {
    NoteRecord {
        meditation_name: NAME.into(),
        meditation_duration_ms: DURATION_MS,
        text: text.into(),
        created_at: ts(3),
    }
}

fn playback(position_ms: i64, completed: bool) -> PlaybackRecord {
    PlaybackRecord {
        meditation_name: NAME.into(),
        meditation_duration_ms: DURATION_MS,
        position_ms,
        completed,
    }
}

async fn run(
    incoming: PracticeSnapshot,
    existing: PracticeSnapshot,
) -> (ImportReport, Vec<String>) {
    let meditations = library();
    let index = meditation_index(&meditations);
    let sink = FakePracticeSink::default();
    let mut report = ImportReport::default();
    apply_practice(user(), incoming, &existing, &index, &sink, &mut report)
        .await
        .unwrap();
    let calls = sink.calls.lock().unwrap().clone();
    (report, calls)
}

#[test]
fn disposition_applies_linked_non_duplicate() {
    assert!(matches!(disposition(Some(4), false), Disposition::Apply(4)));
}

#[test]
fn disposition_flags_linked_duplicate() {
    assert!(matches!(disposition(Some(4), true), Disposition::Duplicate));
}

#[test]
fn disposition_unlinked_wins_over_duplicate_flag() {
    assert!(matches!(disposition(None, true), Disposition::Unlinked));
    assert!(matches!(disposition(None, false), Disposition::Unlinked));
}

#[test]
fn tally_increments_matching_counter_only() {
    let mut report = ImportReport::default();
    tally(&Disposition::Apply(1), &mut report);
    tally(&Disposition::Duplicate, &mut report);
    tally(&Disposition::Duplicate, &mut report);
    tally(&Disposition::Unlinked, &mut report);
    assert_eq!(report.practice_linked, 1);
    assert_eq!(report.practice_duplicates_skipped, 2);
    assert_eq!(report.practice_skipped, 1);
    assert_eq!(report.mood_created, 0);
}

#[test]
fn classify_returns_id_for_new_linked_key() {
    let mut report = ImportReport::default();
    let keys: HashSet<u8> = HashSet::new();
    assert_eq!(classify(&keys, &1, Some(9), &mut report), Some(9));
    assert_eq!(report.practice_linked, 1);
}

#[test]
fn classify_returns_none_for_known_key_or_missing_link() {
    let mut report = ImportReport::default();
    let keys = HashSet::from([1u8]);
    assert_eq!(classify(&keys, &1, Some(9), &mut report), None);
    assert_eq!(classify(&keys, &2, None, &mut report), None);
    assert_eq!(report.practice_duplicates_skipped, 1);
    assert_eq!(report.practice_skipped, 1);
    assert_eq!(report.practice_linked, 0);
}

#[tokio::test]
async fn apply_writes_linked_records_to_sink() {
    let incoming = snapshot(
        vec![session(NAME, 10)],
        vec![note("hi")],
        vec![playback(250, false)],
    );

    let (report, calls) = run(incoming, PracticeSnapshot::default()).await;

    assert_eq!(report.practice_linked, 3);
    assert_eq!(calls, vec!["session:5", "note:5", "position:5"]);
}

#[tokio::test]
async fn apply_marks_completed_only_for_completed_playback() {
    let incoming = snapshot(vec![], vec![], vec![playback(900, true)]);

    let (_, calls) = run(incoming, PracticeSnapshot::default()).await;

    assert_eq!(calls, vec!["position:5", "completed:5"]);
}

#[tokio::test]
async fn apply_skips_records_for_unknown_meditations() {
    let incoming = snapshot(vec![session("Unknown", 10)], vec![], vec![]);

    let (report, calls) = run(incoming, PracticeSnapshot::default()).await;

    assert_eq!(report.practice_skipped, 1);
    assert_eq!(report.practice_linked, 0);
    assert!(calls.is_empty());
}

#[tokio::test]
async fn apply_skips_records_already_present() {
    let existing_session = session(NAME, 10);
    let existing_note = note("hi");
    let existing_playback = playback(250, true);
    let incoming = snapshot(
        vec![existing_session.clone()],
        vec![existing_note.clone()],
        vec![existing_playback.clone()],
    );
    let existing = snapshot(
        vec![existing_session],
        vec![existing_note],
        vec![existing_playback],
    );

    let (report, calls) = run(incoming, existing).await;

    assert_eq!(report.practice_duplicates_skipped, 3);
    assert!(calls.is_empty());
}

#[tokio::test]
async fn apply_treats_changed_note_text_or_position_as_new() {
    let incoming = snapshot(vec![], vec![note("new text")], vec![playback(251, false)]);
    let existing = snapshot(vec![], vec![note("old text")], vec![playback(250, false)]);

    let (report, calls) = run(incoming, existing).await;

    assert_eq!(report.practice_linked, 2);
    assert_eq!(calls, vec!["note:5", "position:5"]);
}

#[tokio::test]
async fn apply_does_not_dedupe_repeats_inside_the_import() {
    let repeated = session(NAME, 10);
    let incoming = snapshot(vec![repeated.clone(), repeated], vec![], vec![]);

    let (report, calls) = run(incoming, PracticeSnapshot::default()).await;

    assert_eq!(report.practice_linked, 2);
    assert_eq!(calls, vec!["session:5", "session:5"]);
}
