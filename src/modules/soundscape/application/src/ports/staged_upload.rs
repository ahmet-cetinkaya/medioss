// Mirrors meditation-application's staged_upload port: the boundary rule forbids sharing it across modules.
use async_trait::async_trait;

/// An upload that already lives on disk. `address` is a file name inside
/// the staging directory and is opaque to the application layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedMedia {
    pub address: String,
    pub size_bytes: u64,
}

/// Why staging an upload failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StagingError {
    #[error("staging io failed: {0}")]
    Io(String),
    #[error("free space cannot be measured on this platform")]
    Unsupported,
}

/// Writes one upload to disk chunk by chunk. Dropping a sink that was not
/// kept deletes its file, which covers cap overflow, validation failure and
/// client disconnects.
#[async_trait]
pub trait StagedSink: Send {
    /// Appends `chunk`; a full disk surfaces as [`StagingError::Io`].
    async fn append(&mut self, chunk: &[u8]) -> Result<(), StagingError>;
    /// Flushes and closes the file, keeping it on disk.
    async fn keep(self: Box<Self>) -> Result<StagedMedia, StagingError>;
}

/// Disk staging for uploads, so a large file never sits in memory.
#[async_trait]
pub trait UploadStaging: Send + Sync {
    /// Free bytes on the staging volume.
    async fn free_bytes(&self) -> Result<u64, StagingError>;
    /// Opens a fresh, uniquely named staging file.
    async fn open(&self) -> Result<Box<dyn StagedSink>, StagingError>;
    /// Reads at most `max_bytes` from the start of a staged file.
    async fn read_prefix(
        &self,
        media: &StagedMedia,
        max_bytes: usize,
    ) -> Result<Vec<u8>, StagingError>;
    /// Deletes a kept staged file; an absent file is not an error. Callers use
    /// it on every path that does not hand the file to the blob store.
    async fn discard(&self, media: &StagedMedia) -> Result<(), StagingError>;
}
