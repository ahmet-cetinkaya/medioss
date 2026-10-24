use super::*;
use crate::models::practice_snapshot::PracticeSnapshot;
use chrono::Utc;

#[test]
fn string_converts_to_internal_failure() {
    let failure = ImportFailure::from("db down".to_string());
    assert!(matches!(failure, ImportFailure::Internal(ref m) if m == "db down"));
}

#[test]
fn display_prints_validation_message_verbatim() {
    let failure = ImportFailure::Validation("bad score".into());
    assert_eq!(failure.to_string(), "bad score");
}

#[test]
fn display_prints_internal_message_verbatim() {
    let failure = ImportFailure::Internal("db down".into());
    assert_eq!(failure.to_string(), "db down");
}

#[test]
fn command_carries_user_and_document() {
    let command = ImportOwnData {
        user_id: UserId::new(3),
        document: ExportDocument {
            format_version: 1,
            exported_at: Utc::now(),
            display_name: None,
            mood_entries: vec![],
            practice: PracticeSnapshot::default(),
        },
    };
    assert_eq!(command.user_id, UserId::new(3));
    assert_eq!(command.document.format_version, 1);
}
