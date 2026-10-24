use async_trait::async_trait;
use soundscape_application::ports::media::{BlobReader, BlobStore};

/// keeps the original upload format, so the answer follows the file.
fn content_type_for(key: &str) -> &'static str {
    match key.rsplit('.').next().unwrap_or_default() {
        "opus" => "audio/ogg",
        "flac" => "audio/flac",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

pub struct MediaStoreBlobs(pub media_store::MediaStore);
#[async_trait]
impl BlobStore for MediaStoreBlobs {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        self.0
            .put(folder, stem, ext, bytes)
            .await
            .map(|x| x.as_str().to_owned())
            .map_err(|e| e.to_string())
    }
    async fn open_range(
        &self,
        key: &str,
        start: u64,
        length: u64,
    ) -> Result<(BlobReader, String, u64), String> {
        let key = media_store::StorageKey::parse_with_separator(key).map_err(|e| e.to_string())?;
        let size = self.0.size(&key).await.map_err(|e| e.to_string())?;
        let stream = self
            .0
            .open_range(&key, start, length)
            .await
            .map_err(|e| e.to_string())?;
        let content_type = content_type_for(key.as_str());
        use tokio_util::compat::TokioAsyncReadCompatExt;
        Ok((
            BlobReader(Box::new(stream.compat())),
            content_type.to_owned(),
            size,
        ))
    }
    async fn delete(&self, key: &str) -> Result<(), String> {
        let key = media_store::StorageKey::parse_with_separator(key).map_err(|e| e.to_string())?;
        self.0.delete(&key).await.map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[path = "media_store_blobs.tests.rs"]
mod tests;
