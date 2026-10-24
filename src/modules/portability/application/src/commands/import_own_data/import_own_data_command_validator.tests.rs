use crate::models::note_record::NoteRecord;
use crate::models::practice_snapshot::PracticeSnapshot;
use crate::test_support::fixtures::{document, entry, snapshot, ts};

use super::*;
use kernel::UserId;

fn validate_document(document: &ExportDocument) -> Result<(), ImportFailure> {
    ImportOwnDataCommandValidator.validate(&ImportOwnDataCommand {
        user_id: UserId::new(1),
        document: document.clone(),
    })
}

#[test]
fn valid_document_passes() {
    assert!(validate_document(&document(
        vec![entry(3, Some("ok"), 10)],
        PracticeSnapshot::default()
    ))
    .is_ok());
}

#[test]
fn score_outside_range_rejects() {
    let err = validate_document(&document(
        vec![entry(9, None, 10)],
        PracticeSnapshot::default(),
    ))
    .unwrap_err();
    assert!(err.to_string().contains("outside 1-5"), "{err}");
}

#[test]
fn unknown_format_version_rejects() {
    let mut doc = document(vec![entry(3, None, 10)], PracticeSnapshot::default());
    doc.format_version = 99;
    let err = validate_document(&doc).unwrap_err();
    assert!(err.to_string().contains("format version"), "{err}");
}

#[test]
fn future_mood_entry_rejects() {
    let err = validate_document(&document(
        vec![entry(3, None, -60)],
        PracticeSnapshot::default(),
    ))
    .unwrap_err();
    assert!(err.to_string().contains("future"), "{err}");
}

#[test]
fn oversized_mood_note_rejects() {
    let big = "x".repeat(10_001);
    let err = validate_document(&document(
        vec![entry(3, Some(&big), 10)],
        PracticeSnapshot::default(),
    ))
    .unwrap_err();
    assert!(err.to_string().contains("10000"), "{err}");
}

#[test]
fn empty_practice_note_rejects() {
    let doc = document(
        vec![],
        snapshot(
            vec![],
            vec![NoteRecord {
                meditation_name: "Evening Calm".into(),
                meditation_duration_ms: 300_000,
                text: "   ".into(),
                created_at: ts(60),
            }],
            vec![],
        ),
    );
    let err = validate_document(&doc).unwrap_err();
    assert!(err.to_string().contains("not be empty"), "{err}");
}
