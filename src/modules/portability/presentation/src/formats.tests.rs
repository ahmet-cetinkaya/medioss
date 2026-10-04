use crate::export_format;
use crate::mood_csv;
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
fn csv_round_trip_handles_quotes_commas_newlines_and_unicode() {
    let entries = vec![
        MoodRecord {
            score: 3,
            note: Some("comma, quote \" and\nnewline".into()),
            recorded_at: chrono::Utc::now(),
        },
        MoodRecord {
            score: 5,
            note: None,
            recorded_at: chrono::Utc::now(),
        },
        MoodRecord {
            score: 1,
            note: Some("çalışma ünlem!".into()),
            recorded_at: chrono::Utc::now(),
        },
    ];
    let bytes = mood_csv::to_csv(&entries).unwrap();
    let back = mood_csv::mood_records_from_csv(&bytes).unwrap();
    assert_eq!(back.len(), 3);
    for (a, b) in entries.iter().zip(back.iter()) {
        assert_eq!(a.score, b.score);
        assert_eq!(a.note, b.note);
        assert_eq!(a.recorded_at, b.recorded_at);
    }
}

#[test]
fn csv_with_wrong_column_count_is_rejected() {
    let bytes = b"recorded_at,score,note,extra\n2026-01-01T00:00:00Z,3,note,x\n";
    let err = mood_csv::mood_records_from_csv(bytes).unwrap_err();
    assert!(err.contains("columns"), "{err}");
}

#[test]
fn csv_with_invalid_score_is_rejected() {
    let bytes = b"recorded_at,score,note\n2026-01-01T00:00:00Z,nine,note\n";
    assert!(mood_csv::mood_records_from_csv(bytes).is_err());
}

#[test]
fn malformed_json_is_rejected() {
    let err = export_format::from_json(b"{not json").unwrap_err();
    assert!(err.contains("not a valid export"), "{err}");
}
