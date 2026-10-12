use std::path::PathBuf;

use async_trait::async_trait;
use identity_application::ports::{AvatarStore, AvatarStoreError};
use identity_domain::AvatarRef;
use kernel::UserId;
use rand::Rng;

pub struct FileAvatarStore {
    media_root: PathBuf,
}

const MAX_AVATAR_DIMENSION: u32 = 256;
const MAX_AVATAR_OUTPUT_BYTES: usize = 2 * 1024 * 1024;

impl FileAvatarStore {
    pub fn new(media_root: PathBuf) -> Self {
        Self { media_root }
    }

    fn path_for(&self, id: UserId, avatar: &AvatarRef) -> PathBuf {
        self.media_root
            .join(id.value().to_string())
            .join(avatar.key())
    }

    fn new_avatar(extension: &'static str) -> Result<AvatarRef, AvatarStoreError> {
        let key = format!("avatar-{:016x}.{extension}", rand::rng().random::<u64>());
        AvatarRef::new(key, extension).map_err(|_| AvatarStoreError::Storage)
    }
}

#[async_trait]
impl AvatarStore for FileAvatarStore {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, &'static str), AvatarStoreError> {
        let normalized = crate::image_normalization::normalize(
            bytes,
            crate::image_normalization::ImageSpec {
                maximum_dimension: MAX_AVATAR_DIMENSION,
                maximum_output_bytes: MAX_AVATAR_OUTPUT_BYTES,
                square: true,
            },
        )
        .await
        .map_err(|error| match error {
            crate::image_normalization::ImageNormalizationError::InvalidPayload => {
                AvatarStoreError::InvalidPayload
            }
            crate::image_normalization::ImageNormalizationError::OutputTooLarge => {
                AvatarStoreError::OutputTooLarge
            }
            crate::image_normalization::ImageNormalizationError::WorkerFailed => {
                AvatarStoreError::WorkerFailed
            }
            crate::image_normalization::ImageNormalizationError::Busy => AvatarStoreError::Busy,
        })?;
        Ok((normalized, "webp"))
    }

    async fn write(
        &self,
        id: UserId,
        extension: &'static str,
        bytes: &[u8],
    ) -> Result<AvatarRef, AvatarStoreError> {
        let avatar = Self::new_avatar(extension)?;
        let path = self.path_for(id, &avatar);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| AvatarStoreError::Storage)?;
        }
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|_| AvatarStoreError::Storage)?;
        Ok(avatar)
    }

    async fn read(&self, id: UserId, avatar: &AvatarRef) -> Result<Vec<u8>, AvatarStoreError> {
        tokio::fs::read(self.path_for(id, avatar))
            .await
            .map_err(|_| AvatarStoreError::Storage)
    }

    async fn remove(&self, id: UserId, avatar: &AvatarRef) -> Result<(), AvatarStoreError> {
        let path = self.path_for(id, avatar);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(AvatarStoreError::Storage),
        }
    }
}

#[cfg(test)]
#[path = "avatar_store.tests.rs"]
mod tests;
