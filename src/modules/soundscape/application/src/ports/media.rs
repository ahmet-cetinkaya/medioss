use async_trait::async_trait;
use futures_io::AsyncRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    OggOpus,
    OggVorbis,
}

/// The stored form the converter produced, so callers persist the right
/// extension and content type for the bytes they were handed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredAudio {
    pub extension: &'static str,
    pub content_type: &'static str,
}
#[async_trait]
pub trait AudioValidator: Send + Sync {
    async fn sniff(&self, bytes: &[u8]) -> Result<AudioFormat, ()>;
}
#[async_trait]
pub trait OpusConverter: Send + Sync {
    async fn convert(
        &self,
        bytes: Vec<u8>,
        format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), String>;
}
pub struct BlobReader(pub Box<dyn AsyncRead + Send + Unpin>);

#[async_trait]
pub trait BlobStore: Send + Sync {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<String, String>;
    async fn open_range(
        &self,
        key: &str,
        start: u64,
        length: u64,
    ) -> Result<(BlobReader, String, u64), String>;
    async fn delete(&self, key: &str) -> Result<(), String>;
}
