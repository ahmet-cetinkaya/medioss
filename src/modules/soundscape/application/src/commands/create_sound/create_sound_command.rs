pub const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct CreateSoundCommand {
    pub name: String,
    pub category_id: i64,
    pub bytes: Vec<u8>,
    pub icon_key: Option<String>,
    pub icon: Option<Vec<u8>>,
}
