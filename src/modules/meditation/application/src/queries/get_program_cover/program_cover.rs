/// The cover bytes plus the sniffed content type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCover {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}
