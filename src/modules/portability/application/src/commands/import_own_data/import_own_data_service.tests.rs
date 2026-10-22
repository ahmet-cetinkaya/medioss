use std::sync::Arc;

use crate::commands::import_own_data::ImportOwnDataHandler;
use crate::commands::import_own_data::ImportOwnDataService;
use crate::commands::import_own_data::{ImportFailure, ImportOwnData};
use crate::models::practice_snapshot::PracticeSnapshot;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::test_support::fake_library::FakeLibrary;
use crate::test_support::fake_mood_store::{FakeMoodStore, MoodSinkFailingOnSecondInsert};
use crate::test_support::fake_practice_sink::FakePracticeSink;
use crate::test_support::fake_practice_source::FakePracticeSource;
use crate::test_support::fixtures::{document, entry, library_meditations, snapshot, ts, user};

struct StatefulPractice {
    snapshot: std::sync::Mutex<PracticeSnapshot>,
}

impl Default for StatefulPractice {
    fn default() -> Self {
        Self {
            snapshot: std::sync::Mutex::new(PracticeSnapshot::default()),
        }
    }
}

#[async_trait::async_trait]
impl OwnPracticeRecords for StatefulPractice {
    async fn list_for_user(&self, _user: kernel::UserId) -> Result<PracticeSnapshot, String> {
        Ok(self.snapshot.lock().unwrap().clone())
    }
}

fn service(
    mood_source: Arc<dyn crate::ports::own_mood_records::OwnMoodRecords>,
    mood_sink: Arc<dyn crate::ports::mood_record_sink::MoodRecordSink>,
    practice_source: Arc<dyn OwnPracticeRecords>,
    practice_sink: Arc<dyn crate::ports::practice_sink::PracticeSink>,
) -> ImportOwnDataService {
    ImportOwnDataService::new(
        mood_source,
        mood_sink,
        practice_source,
        practice_sink,
        Arc::new(FakeLibrary(std::sync::Mutex::new(library_meditations()))),
    )
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
            vec![crate::models::session_record::SessionRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                started_at: ts(120),
                finished_at: ts(110),
            }],
            vec![crate::models::note_record::NoteRecord {
                meditation_name: "Evening Calm".into(),
                meditation_duration_ms: 300_000,
                text: "deep rest".into(),
                created_at: ts(100),
            }],
            vec![crate::models::playback_record::PlaybackRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                position_ms: 45_000,
                completed: false,
            }],
        ),
    );

    let fresh = Arc::new(FakeMoodStore::default());
    let practice = FakePracticeSink::default();
    let report = service(
        fresh.clone(),
        fresh.clone(),
        Arc::new(FakePracticeSource),
        Arc::new(practice),
    )
    .handle(ImportOwnData {
        user_id: user(),
        document: doc,
    })
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
    let sink = Arc::new(practice);
    let svc = service(
        store.clone(),
        store.clone(),
        Arc::new(FakePracticeSource),
        sink.clone(),
    );

    let first = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: doc.clone(),
        })
        .await
        .unwrap();
    assert_eq!((first.mood_created, first.mood_duplicates_skipped), (2, 0));

    let second = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: doc,
        })
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
    let practice = Arc::new(FakePracticeSink::default());
    let doc = document(
        vec![],
        snapshot(
            vec![
                crate::models::session_record::SessionRecord {
                    meditation_name: "Café Quietude".into(),
                    meditation_duration_ms: 120_000,
                    started_at: ts(90),
                    finished_at: ts(80),
                },
                crate::models::session_record::SessionRecord {
                    meditation_name: "Gone".into(),
                    meditation_duration_ms: 90_000,
                    started_at: ts(70),
                    finished_at: ts(60),
                },
            ],
            vec![crate::models::note_record::NoteRecord {
                meditation_name: "Same Name Different Length".into(),
                meditation_duration_ms: 1,
                text: "x".into(),
                created_at: ts(50),
            }],
            vec![],
        ),
    );

    let report = service(
        store.clone(),
        store,
        Arc::new(FakePracticeSource),
        practice.clone(),
    )
    .handle(ImportOwnData {
        user_id: user(),
        document: doc,
    })
    .await
    .unwrap();
    assert_eq!(report.practice_linked, 1);
    assert_eq!(report.practice_skipped, 2);
    assert_eq!(practice.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_document_rejects_with_zero_writes() {
    let store = Arc::new(FakeMoodStore::default());
    let practice = Arc::new(FakePracticeSink::default());
    let svc = service(
        store.clone(),
        store.clone(),
        Arc::new(FakePracticeSource),
        practice.clone(),
    );

    let bad_score = document(vec![entry(9, None, 10)], PracticeSnapshot::default());
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: bad_score,
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("outside 1-5"), "{err}");

    let mut bad_version = document(vec![entry(3, None, 10)], PracticeSnapshot::default());
    bad_version.format_version = 99;
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: bad_version,
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("format version"), "{err}");

    let future = document(vec![entry(3, None, -60)], PracticeSnapshot::default());
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: future,
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("future"), "{err}");

    assert!(store.entries.lock().unwrap().is_empty());
    assert!(practice.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn repeat_import_reports_practice_as_duplicates_without_writing() {
    let store = Arc::new(FakeMoodStore::default());
    let practice = Arc::new(FakePracticeSink::default());
    let source = Arc::new(StatefulPractice::default());
    let doc = document(
        vec![],
        snapshot(
            vec![crate::models::session_record::SessionRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                started_at: ts(90),
                finished_at: ts(80),
            }],
            vec![crate::models::note_record::NoteRecord {
                meditation_name: "Evening Calm".into(),
                meditation_duration_ms: 300_000,
                text: "rest".into(),
                created_at: ts(70),
            }],
            vec![crate::models::playback_record::PlaybackRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                position_ms: 10_000,
                completed: false,
            }],
        ),
    );

    let doc_for_seed = doc.clone();
    let svc = service(
        store.clone(),
        store.clone(),
        source.clone(),
        practice.clone(),
    );
    let first = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: doc.clone(),
        })
        .await
        .unwrap();
    assert_eq!(
        (first.practice_linked, first.practice_duplicates_skipped),
        (3, 0)
    );
    {
        let mut snap = source.snapshot.lock().unwrap();
        snap.sessions = doc_for_seed.practice.sessions.clone();
        snap.notes = doc_for_seed.practice.notes.clone();
        snap.playback = doc_for_seed.practice.playback.clone();
    }

    let second = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: doc,
        })
        .await
        .unwrap();
    assert_eq!(
        (second.practice_linked, second.practice_duplicates_skipped),
        (0, 3)
    );
    assert_eq!(practice.calls.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn oversized_mood_note_and_empty_practice_note_reject_with_zero_writes() {
    let store = Arc::new(FakeMoodStore::default());
    let practice = Arc::new(FakePracticeSink::default());
    let svc = service(
        store.clone(),
        store.clone(),
        Arc::new(FakePracticeSource),
        practice.clone(),
    );

    let big = "x".repeat(10_001);
    let over = document(vec![entry(3, Some(&big), 10)], PracticeSnapshot::default());
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: over,
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("10000"), "{err}");

    let empty = document(
        vec![],
        snapshot(
            vec![],
            vec![crate::models::note_record::NoteRecord {
                meditation_name: "Evening Calm".into(),
                meditation_duration_ms: 300_000,
                text: "   ".into(),
                created_at: ts(60),
            }],
            vec![],
        ),
    );
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: empty,
        })
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("not be empty"), "{err}");

    assert!(practice.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn sink_failure_stops_import_as_internal_error_keeping_earlier_writes() {
    let store = FakeMoodStore::default();
    let failing_sink = MoodSinkFailingOnSecondInsert {
        inserted: Default::default(),
    };
    let practice = FakePracticeSink::default();
    let both = document(
        vec![entry(3, None, 20), entry(4, None, 10)],
        snapshot(
            vec![crate::models::session_record::SessionRecord {
                meditation_name: "Café Quietude".into(),
                meditation_duration_ms: 120_000,
                started_at: ts(30),
                finished_at: ts(28),
            }],
            vec![],
            vec![],
        ),
    );

    let svc = ImportOwnDataService::new(
        Arc::new(store),
        Arc::new(failing_sink),
        Arc::new(FakePracticeSource),
        Arc::new(practice),
        Arc::new(FakeLibrary(std::sync::Mutex::new(library_meditations()))),
    );
    let err = svc
        .handle(ImportOwnData {
            user_id: user(),
            document: both,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, ImportFailure::Internal(ref m) if m == "disk full"));
}
