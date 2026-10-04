use crate::models::export_document::ExportDocument;
use crate::models::mood_record::MoodRecord;
use crate::models::note_record::NoteRecord;
use crate::models::playback_record::PlaybackRecord;
use crate::models::practice_snapshot::PracticeSnapshot;
use crate::models::session_record::SessionRecord;
use crate::ports::meditation_lookup::{MeditationLookup, MeditationRef};
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::practice_sink::PracticeSink;
use chrono::{DateTime, Utc};
use kernel::UserId;
use std::sync::Arc;

fn ts(minutes_ago: i64) -> DateTime<Utc> {
    Utc::now() - chrono::Duration::minutes(minutes_ago)
}

fn entry(score: u8, note: Option<&str>, minutes_ago: i64) -> MoodRecord {
    MoodRecord {
        score,
        note: note.map(String::from),
        recorded_at: ts(minutes_ago),
    }
}

fn snapshot(
    sessions: Vec<SessionRecord>,
    notes: Vec<NoteRecord>,
    playback: Vec<PlaybackRecord>,
) -> PracticeSnapshot {
    PracticeSnapshot {
        sessions,
        notes,
        playback,
    }
}

fn document(mood: Vec<MoodRecord>, practice: PracticeSnapshot) -> ExportDocument {
    ExportDocument {
        format_version: 1,
        exported_at: Utc::now(),
        display_name: Some("Test User".into()),
        mood_entries: mood,
        practice,
    }
}

#[derive(Default)]
struct FakeMoodStore {
    entries: std::sync::Mutex<Vec<MoodRecord>>,
}

#[async_trait::async_trait]
impl OwnMoodRecords for FakeMoodStore {
    async fn list_for_user(&self, _user: UserId) -> Result<Vec<MoodRecord>, String> {
        Ok(self.entries.lock().unwrap().clone())
    }
}

#[async_trait::async_trait]
impl MoodRecordSink for FakeMoodStore {
    async fn insert(&self, _user: UserId, record: MoodRecord) -> Result<(), String> {
        self.entries.lock().unwrap().push(record);
        Ok(())
    }
}

#[derive(Default)]
struct FakePracticeSink {
    calls: std::sync::Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl PracticeSink for FakePracticeSink {
    async fn record_session(
        &self,
        _u: UserId,
        id: i64,
        _s: DateTime<Utc>,
        _f: DateTime<Utc>,
    ) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("session:{id}"));
        Ok(())
    }
    async fn insert_note(
        &self,
        _u: UserId,
        id: i64,
        _t: &str,
        _c: DateTime<Utc>,
    ) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("note:{id}"));
        Ok(())
    }
    async fn upsert_position(&self, _u: UserId, id: i64, _p: i64) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("position:{id}"));
        Ok(())
    }
    async fn set_completed(&self, _u: UserId, id: i64, _c: bool) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("completed:{id}"));
        Ok(())
    }
}

#[derive(Default)]
struct FakeProfile;

#[async_trait::async_trait]
impl crate::ports::display_name_resolver::DisplayNameResolver for FakeProfile {
    async fn display_name(&self, _user: UserId) -> Result<Option<String>, String> {
        Ok(Some("Test User".into()))
    }
}

struct FakePracticeSource;

#[async_trait::async_trait]
impl OwnPracticeRecords for FakePracticeSource {
    async fn list_for_user(&self, _user: UserId) -> Result<PracticeSnapshot, String> {
        Ok(PracticeSnapshot::default())
    }
}

struct FakeLibrary(Vec<MeditationRef>);

#[async_trait::async_trait]
impl MeditationLookup for FakeLibrary {
    async fn list_all(&self) -> Result<Vec<MeditationRef>, String> {
        Ok(self.0.clone())
    }
}

fn library() -> FakeLibrary {
    FakeLibrary(vec![
        MeditationRef {
            id: 1,
            name: "Café Quietude".into(),
            duration_ms: 120_000,
        },
        MeditationRef {
            id: 2,
            name: "Evening Calm".into(),
            duration_ms: 300_000,
        },
    ])
}

fn user() -> UserId {
    UserId::new(7)
}

#[tokio::test]
async fn round_trip_reproduces_history_exactly() {
    let mood = Arc::new(FakeMoodStore::default());
    let source_entries = vec![
        entry(4, Some("calm morning, café & \"focus\""), 600),
        entry(2, None, 300),
        entry(5, Some("ünlem: çalışma!"), 60),
    ];
    for e in &source_entries {
        MoodRecordSink::insert(mood.as_ref(), user(), e.clone())
            .await
            .unwrap();
    }

    let doc = document(
        source_entries.clone(),
        snapshot(
            vec![SessionRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                started_at: ts(120),
                finished_at: ts(110),
            }],
            vec![NoteRecord {
                meditation_name: "Evening Calm".into(),
                meditation_duration_ms: 300_000,
                text: "deep rest".into(),
                created_at: ts(100),
            }],
            vec![PlaybackRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                position_ms: 45_000,
                completed: false,
            }],
        ),
    );

    let fresh = Arc::new(FakeMoodStore::default());
    let practice = FakePracticeSink::default();
    let report = crate::import::import_document(
        user(),
        doc,
        fresh.as_ref(),
        fresh.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap();

    assert_eq!(report.mood_created, 3);
    assert_eq!(report.mood_duplicates_skipped, 0);
    assert_eq!(report.practice_linked, 3);
    assert_eq!(report.practice_skipped, 0);
    let restored = fresh.entries.lock().unwrap().clone();
    assert_eq!(restored.len(), 3);
    for e in &source_entries {
        assert!(restored
            .iter()
            .any(|r| r.score == e.score && r.note == e.note && r.recorded_at == e.recorded_at));
    }
}

#[tokio::test]
async fn second_import_of_same_file_creates_nothing() {
    let store = Arc::new(FakeMoodStore::default());
    let doc = document(
        vec![entry(3, Some("dup"), 500), entry(4, None, 400)],
        PracticeSnapshot::default(),
    );
    let practice = FakePracticeSink::default();

    let first = crate::import::import_document(
        user(),
        doc.clone(),
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap();
    assert_eq!((first.mood_created, first.mood_duplicates_skipped), (2, 0));

    let second = crate::import::import_document(
        user(),
        doc,
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap();
    assert_eq!(
        (second.mood_created, second.mood_duplicates_skipped),
        (0, 2)
    );
    assert_eq!(store.entries.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn practice_records_for_missing_meditations_are_skipped_and_counted() {
    let store = Arc::new(FakeMoodStore::default());
    let practice = FakePracticeSink::default();
    let doc = document(
        vec![],
        snapshot(
            vec![
                SessionRecord {
                    meditation_name: "Café Quietude".into(),
                    meditation_duration_ms: 120_000,
                    started_at: ts(90),
                    finished_at: ts(80),
                },
                SessionRecord {
                    meditation_name: "Gone".into(),
                    meditation_duration_ms: 90_000,
                    started_at: ts(70),
                    finished_at: ts(60),
                },
            ],
            vec![NoteRecord {
                meditation_name: "Same Name Different Length".into(),
                meditation_duration_ms: 1,
                text: "x".into(),
                created_at: ts(50),
            }],
            vec![],
        ),
    );

    let report = crate::import::import_document(
        user(),
        doc,
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap();
    assert_eq!(report.practice_linked, 1);
    assert_eq!(report.practice_skipped, 2);
    assert_eq!(practice.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_document_rejects_with_zero_writes() {
    let store = Arc::new(FakeMoodStore::default());
    let practice = FakePracticeSink::default();

    let bad_score = document(vec![entry(9, None, 10)], PracticeSnapshot::default());
    let err = crate::import::import_document(
        user(),
        bad_score,
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap_err();
    assert!(err.contains("outside 1-5"), "{err}");

    let mut bad_version = document(vec![entry(3, None, 10)], PracticeSnapshot::default());
    bad_version.format_version = 99;
    let err = crate::import::import_document(
        user(),
        bad_version,
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap_err();
    assert!(err.contains("format version"), "{err}");

    let future = document(vec![entry(3, None, -60)], PracticeSnapshot::default());
    let err = crate::import::import_document(
        user(),
        future,
        store.as_ref(),
        store.as_ref(),
        &practice,
        &library(),
    )
    .await
    .unwrap_err();
    assert!(err.contains("future"), "{err}");

    assert!(store.entries.lock().unwrap().is_empty());
    assert!(practice.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn export_lists_own_records_through_ports() {
    let mood = Arc::new(FakeMoodStore::default());
    MoodRecordSink::insert(mood.as_ref(), user(), entry(5, Some("great"), 30))
        .await
        .unwrap();
    let doc =
        crate::export::export_document(user(), mood.as_ref(), &FakePracticeSource, &FakeProfile)
            .await
            .unwrap();
    assert_eq!(doc.format_version, 1);
    assert_eq!(doc.mood_entries.len(), 1);
    assert_eq!(doc.mood_entries[0].score, 5);
}
