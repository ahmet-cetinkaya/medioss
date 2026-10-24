use std::sync::Mutex;

use async_trait::async_trait;
use identity_domain::AvatarRef;
use kernel::UserId;

use crate::ports::{AvatarStore, AvatarStoreError};

/// Records every store interaction and can be told to reject normalization
/// or fail writes, mirroring the fixture the avatar commands were tested
/// with before the use-case layout.
pub(crate) struct FakeAvatarStore {
    pub(crate) normalize_result: Mutex<Result<(Vec<u8>, &'static str), AvatarStoreError>>,
    pub(crate) written: Mutex<Vec<(i64, &'static str)>>,
    pub(crate) removed: Mutex<Vec<(i64, &'static str)>>,
    next_key: std::sync::atomic::AtomicU64,
    fail_writes: std::sync::atomic::AtomicBool,
}

impl FakeAvatarStore {
    pub(crate) fn accepting() -> Self {
        Self {
            normalize_result: Mutex::new(Ok((vec![1, 2, 3], "webp"))),
            written: Mutex::new(Vec::new()),
            removed: Mutex::new(Vec::new()),
            next_key: std::sync::atomic::AtomicU64::new(1),
            fail_writes: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn rejecting_with(error: AvatarStoreError) -> Self {
        Self {
            normalize_result: Mutex::new(Err(error)),
            ..Self::accepting()
        }
    }

    pub(crate) fn failing_writes() -> Self {
        Self {
            fail_writes: std::sync::atomic::AtomicBool::new(true),
            ..Self::accepting()
        }
    }

    pub(crate) fn written(&self) -> Vec<(i64, &'static str)> {
        self.written.lock().unwrap().clone()
    }

    pub(crate) fn removed(&self) -> Vec<(i64, &'static str)> {
        self.removed.lock().unwrap().clone()
    }
}

#[async_trait]
impl AvatarStore for FakeAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), AvatarStoreError> {
        self.normalize_result.lock().unwrap().clone()
    }

    async fn write(
        &self,
        id: UserId,
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<AvatarRef, AvatarStoreError> {
        if self.fail_writes.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AvatarStoreError::Storage);
        }
        let sequence = self
            .next_key
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let avatar =
            AvatarRef::new(format!("avatar-{sequence:016x}.{extension}"), extension).unwrap();
        self.written
            .lock()
            .unwrap()
            .push((id.value(), avatar.extension()));
        Ok(avatar)
    }

    async fn read(&self, _id: UserId, _avatar: &AvatarRef) -> Result<Vec<u8>, AvatarStoreError> {
        Ok(vec![9, 9, 9])
    }

    async fn remove(&self, id: UserId, avatar: &AvatarRef) -> Result<(), AvatarStoreError> {
        self.removed
            .lock()
            .unwrap()
            .push((id.value(), avatar.extension()));
        Ok(())
    }
}
