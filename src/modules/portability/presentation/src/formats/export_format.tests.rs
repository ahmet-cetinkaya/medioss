use crate::formats::export_format;
use portability_application::models::export_document::ExportDocument;
use portability_application::models::mood_record::MoodRecord;
use portability_application::models::practice_snapshot::PracticeSnapshot;

fn document(entries: Vec<MoodRecord>) -> ExportDocument {
    ExportDocument {
        format_version: 1,
        exported_at: chrono::Utc::now(),
        display_name: None,
        mood_entries: entries,
        practice: PracticeSnapshot::default(),
    }
}

#[test]
fn json_round_trip_preserves_all_fields() {
    let doc = document(vec![MoodRecord {
        score: 4,
        note: Some("calm, \"quoted\"\nnewline — ünïcode".into()),
        recorded_at: chrono::Utc::now(),
    }]);
    let bytes = export_format::to_json(&doc).unwrap();
    let back = export_format::from_json(&bytes).unwrap();
    assert_eq!(back.format_version, doc.format_version);
    assert_eq!(back.mood_entries.len(), 1);
    assert_eq!(back.mood_entries[0].note, doc.mood_entries[0].note);
    assert_eq!(back.mood_entries[0].score, 4);
}

#[test]
fn json_unknown_version_is_rejected() {
    let mut doc = document(vec![]);
    doc.format_version = 99;
    let bytes = export_format::to_json(&doc).unwrap();
    let err = export_format::from_json(&bytes).unwrap_err();
    assert!(err.contains("format version"), "{err}");
}

#[test]
fn malformed_json_is_rejected() {
    let err = export_format::from_json(b"{not json").unwrap_err();
    assert!(err.contains("not a valid export"), "{err}");
}
