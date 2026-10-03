use std::path::PathBuf;

use async_trait::async_trait;
use identity_application::ports::{AvatarStore, AvatarStoreError};
use identity_domain::AvatarRef;
use kernel::UserId;

pub struct FileAvatarStore {
    media_root: PathBuf,
}

const MAX_AVATAR_DIMENSION: u32 = 256;

impl FileAvatarStore {
    pub fn new(media_root: PathBuf) -> Self {
        Self { media_root }
    }

    fn path_for(&self, id: UserId, avatar: AvatarRef) -> PathBuf {
        self.media_root
            .join(id.value().to_string())
            .join(format!("avatar.{}", avatar.extension()))
    }
}

#[async_trait]
impl AvatarStore for FileAvatarStore {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, &'static str), AvatarStoreError> {
        let format = image::guess_format(&bytes).map_err(|_| AvatarStoreError::InvalidPayload)?;
        if !matches!(
            format,
            image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP
        ) {
            return Err(AvatarStoreError::InvalidPayload);
        }

        let loaded = decode_with_limits(&bytes, format)?;
        let square = tokio::task::spawn_blocking(move || {
            center_square(loaded).resize_to_fill(
                MAX_AVATAR_DIMENSION,
                MAX_AVATAR_DIMENSION,
                image::imageops::FilterType::Lanczos3,
            )
        })
        .await
        .map_err(|_| AvatarStoreError::Storage)?;

        let mut webp = Vec::new();
        if square
            .write_to(
                &mut std::io::Cursor::new(&mut webp),
                image::ImageFormat::WebP,
            )
            .is_ok()
            && !webp.is_empty()
        {
            return Ok((webp, "webp"));
        }
        let mut png = Vec::new();
        square
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|_| AvatarStoreError::InvalidPayload)?;
        Ok((png, "png"))
    }

    async fn write(
        &self,
        id: UserId,
        avatar: AvatarRef,
        bytes: &[u8],
    ) -> Result<(), AvatarStoreError> {
        let path = self.path_for(id, avatar);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| AvatarStoreError::Storage)?;
        }
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|_| AvatarStoreError::Storage)
    }

    async fn read(&self, id: UserId, avatar: AvatarRef) -> Result<Vec<u8>, AvatarStoreError> {
        tokio::fs::read(self.path_for(id, avatar))
            .await
            .map_err(|_| AvatarStoreError::Storage)
    }

    async fn remove(&self, id: UserId, avatar: AvatarRef) -> Result<(), AvatarStoreError> {
        let path = self.path_for(id, avatar);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(AvatarStoreError::Storage),
        }
    }
}

fn center_square(image: image::DynamicImage) -> image::DynamicImage {
    let (width, height) = (image.width(), image.height());
    let side = width.min(height);
    image.crop_imm((width - side) / 2, (height - side) / 2, side, side)
}

const MAX_DECODE_DIMENSION: u32 = 8192;
const MAX_DECODE_ALLOC: u64 = 128 * 1024 * 1024;

fn decode_with_limits(
    bytes: &[u8],
    format: image::ImageFormat,
) -> Result<image::DynamicImage, AvatarStoreError> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AvatarStoreError::InvalidPayload)?;
    if reader.format() != Some(format) {
        return Err(AvatarStoreError::InvalidPayload);
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_DIMENSION);
    limits.max_image_height = Some(MAX_DECODE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|_| AvatarStoreError::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let image = if width * height > 1_000_000 {
            image::DynamicImage::new_luma8(width, height)
        } else {
            image::DynamicImage::new_rgb8(width, height)
        };
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        bytes
    }

    fn store() -> FileAvatarStore {
        FileAvatarStore::new(
            std::env::temp_dir().join(format!("medioss-avatar-test-{}", std::process::id())),
        )
    }

    #[tokio::test]
    async fn normalizes_to_bounded_square_webp() {
        let (encoded, ext) = store().normalize(png_bytes(400, 200)).await.unwrap();
        let reloaded = image::load_from_memory(&encoded).unwrap();
        assert_eq!(ext, "webp");
        assert_eq!(reloaded.width(), 256);
        assert_eq!(reloaded.height(), 256);
    }

    #[tokio::test]
    async fn rejects_non_raster_payloads() {
        let store = store();
        assert_eq!(
            store.normalize(b"<svg/>".to_vec()).await.unwrap_err(),
            AvatarStoreError::InvalidPayload
        );
        assert_eq!(
            store.normalize(b"plain text".to_vec()).await.unwrap_err(),
            AvatarStoreError::InvalidPayload
        );
        assert_eq!(
            store
                .normalize(vec![0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00])
                .await
                .unwrap_err(),
            AvatarStoreError::InvalidPayload
        );
    }

    #[tokio::test]
    async fn rejects_dimension_bombs_before_allocating() {
        let bomb = png_bytes(9_000, 9_000);
        assert_eq!(
            store().normalize(bomb).await.unwrap_err(),
            AvatarStoreError::InvalidPayload
        );
    }

    #[tokio::test]
    async fn write_read_remove_round_trip() {
        let store = store();
        let id = UserId::new(9_999_001);

        store.write(id, AvatarRef::Webp, b"bytes").await.unwrap();
        assert_eq!(store.read(id, AvatarRef::Webp).await.unwrap(), b"bytes");

        store.remove(id, AvatarRef::Webp).await.unwrap();
        store.remove(id, AvatarRef::Webp).await.unwrap();
        assert!(store.read(id, AvatarRef::Webp).await.is_err());
    }
}
