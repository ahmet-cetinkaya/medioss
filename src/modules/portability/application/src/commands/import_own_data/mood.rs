use crate::models::import_report::ImportReport;
use crate::models::mood_record::MoodRecord;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use chrono::{DateTime, Utc};
use kernel::UserId;
use std::collections::HashSet;

pub(super) type MoodKey = (DateTime<Utc>, u8, Option<String>);

pub(super) async fn existing_mood_keys(
    user_id: UserId,
    source: &dyn OwnMoodRecords,
) -> Result<HashSet<MoodKey>, String> {
    Ok(source
        .list_for_user(user_id)
        .await?
        .into_iter()
        .map(|r| (r.recorded_at, r.score, r.note))
        .collect())
}

pub(super) async fn apply_mood_entries(
    user_id: UserId,
    entries: Vec<MoodRecord>,
    mut existing: HashSet<MoodKey>,
    sink: &dyn MoodRecordSink,
    report: &mut ImportReport,
) -> Result<(), String> {
    for entry in entries {
        let key = (entry.recorded_at, entry.score, entry.note.clone());
        if existing.contains(&key) {
            report.mood_duplicates_skipped += 1;
        } else {
            sink.insert(user_id, entry).await?;
            existing.insert(key);
            report.mood_created += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "mood.tests.rs"]
mod tests;
