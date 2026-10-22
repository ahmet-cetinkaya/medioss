use std::sync::Arc;

use crate::queries::export_mood_records::{
    ExportMoodRecords, ExportMoodRecordsHandler, ExportMoodRecordsService,
};
use crate::test_support::fake_mood_store::FakeMoodStore;
use crate::test_support::fixtures::{entry, user};

#[tokio::test]
async fn export_returns_the_mood_records_the_port_lists() {
    let mood = Arc::new(FakeMoodStore::default());
    mood.entries.lock().unwrap().push(entry(2, None, 300));
    mood.entries
        .lock()
        .unwrap()
        .push(entry(4, Some("calm"), 600));

    let service = ExportMoodRecordsService::new(mood);
    let records = service
        .handle(ExportMoodRecords { user_id: user() })
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].score, 2);
    assert_eq!(records[1].note, Some("calm".into()));
}
