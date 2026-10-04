use crate::models::export_document::ExportDocument;
use crate::models::mood_record::MoodRecord;
use crate::ports::display_name_resolver::DisplayNameResolver;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use kernel::UserId;

pub const FORMAT_VERSION: u32 = 1;

pub async fn export_document(
    user_id: UserId,
    mood: &dyn OwnMoodRecords,
    practice: &dyn OwnPracticeRecords,
    profile: &dyn DisplayNameResolver,
) -> Result<ExportDocument, String> {
    Ok(ExportDocument {
        format_version: FORMAT_VERSION,
        exported_at: chrono::Utc::now(),
        display_name: profile.display_name(user_id).await?,
        mood_entries: mood.list_for_user(user_id).await?,
        practice: practice.list_for_user(user_id).await?,
    })
}

pub async fn export_mood_records(
    user_id: UserId,
    mood: &dyn OwnMoodRecords,
) -> Result<Vec<MoodRecord>, String> {
    mood.list_for_user(user_id).await
}
