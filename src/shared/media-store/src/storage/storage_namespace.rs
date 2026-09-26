/// The storage area a key lives in.
///
/// Fieldless on purpose: a namespace becomes a directory name under
/// `$MEDIOSS_DATA_DIR/media/`, so an open `String` would let a caller inject a
/// path. Variants are added when a milestone actually stores blobs; M0 stores
/// meditation audio only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageNamespace {
    Meditation,
    ProgramCover,
}

impl StorageNamespace {
    /// The on-disk directory name for this namespace.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Meditation => "meditation",
            Self::ProgramCover => "program-cover",
        }
    }
}
