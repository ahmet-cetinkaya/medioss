pub const MAX_ICON_BYTES: usize = 1024 * 1024;

/// Stored location of an admin-uploaded icon image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundIcon {
    pub storage_key: String,
    pub content_type: String,
}
