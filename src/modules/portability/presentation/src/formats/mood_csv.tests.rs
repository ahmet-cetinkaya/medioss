use crate::formats::{mood_records_from_csv, to_csv};
use portability_application::models::mood_record::MoodRecord;

fn csv_entries() -> Vec<MoodRecord> {
    vec![
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
    ]
}

#[test]
fn csv_round_trip_handles_quotes_commas_newlines_and_unicode() {
    let entries = csv_entries();
    let bytes = to_csv(&entries).unwrap();
    let back = mood_records_from_csv(&bytes).unwrap();
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
    let err = mood_records_from_csv(bytes).unwrap_err();
    assert!(err.contains("columns"), "{err}");
}

#[test]
fn csv_with_invalid_score_is_rejected() {
    let bytes = b"recorded_at,score,note\n2026-01-01T00:00:00Z,nine,note\n";
    assert!(mood_records_from_csv(bytes).is_err());
}
