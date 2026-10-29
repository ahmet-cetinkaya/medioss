use crate::ports::StagedMedia;

#[derive(Debug, Clone)]
pub struct UploadMeditationCommand {
    pub name: String,
    pub program_id: i64,
    pub media: StagedMedia,
    pub description: Option<String>,
}
