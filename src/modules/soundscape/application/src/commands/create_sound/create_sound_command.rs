use crate::ports::staged_upload::StagedMedia;

#[derive(Debug, Clone)]
pub struct CreateSoundCommand {
    pub name: String,
    pub category_id: i64,
    pub media: StagedMedia,
    pub icon_key: Option<String>,
    pub icon: Option<Vec<u8>>,
}
