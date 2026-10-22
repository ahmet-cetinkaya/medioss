use std::sync::Arc;

use crate::ports::mood_record_sink::MoodRecordSink;
use crate::queries::export_own_data::{ExportOwnData, ExportOwnDataHandler, ExportOwnDataService};
use crate::test_support::fake_mood_store::FakeMoodStore;
use crate::test_support::fake_practice_source::FakePracticeSource;
use crate::test_support::fake_profile::FakeProfile;
use crate::test_support::fixtures::{entry, user};

#[tokio::test]
async fn export_lists_own_records_through_ports() {
    let mood = Arc::new(FakeMoodStore::default());
    MoodRecordSink::insert(mood.as_ref(), user(), entry(5, Some("great"), 30))
        .await
        .unwrap();

    let service =
        ExportOwnDataService::new(mood, Arc::new(FakePracticeSource), Arc::new(FakeProfile));
    let doc = service
        .handle(ExportOwnData { user_id: user() })
        .await
        .unwrap();

    assert_eq!(doc.format_version, 1);
    assert_eq!(doc.mood_entries.len(), 1);
    assert_eq!(doc.mood_entries[0].score, 5);
    assert_eq!(doc.display_name, Some("Test User".into()));
}
