use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ports::{StagedMedia, StagedSink, StagingError, UploadStaging};

const UNLIMITED_FREE_BYTES: u64 = u64::MAX;

#[derive(Default)]
struct StagingState {
    files: HashMap<String, Vec<u8>>,
    deleted: Vec<String>,
    next_id: u64,
}

/// In-memory staging area. Files deleted by a dropped sink or by `discard`
/// are recorded so tests can assert cleanup.
#[derive(Clone)]
pub struct FakeUploadStaging {
    state: Arc<Mutex<StagingState>>,
    free_bytes: Arc<Mutex<Option<u64>>>,
}

impl Default for FakeUploadStaging {
    fn default() -> Self {
        Self {
            state: Arc::default(),
            free_bytes: Arc::new(Mutex::new(Some(UNLIMITED_FREE_BYTES))),
        }
    }
}

impl FakeUploadStaging {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages `bytes` directly, as a finished upload.
    pub fn stage(&self, bytes: &[u8]) -> StagedMedia {
        let mut state = self.state.lock().unwrap();
        let address = Self::next_address(&mut state);
        state.files.insert(address.clone(), bytes.to_vec());
        StagedMedia {
            address,
            size_bytes: bytes.len() as u64,
        }
    }

    /// Sets the reported free space; `None` simulates a platform without it.
    pub fn set_free_bytes(&self, free_bytes: Option<u64>) {
        *self.free_bytes.lock().unwrap() = free_bytes;
    }

    /// Number of staged files currently on the fake disk.
    pub fn file_count(&self) -> usize {
        self.state.lock().unwrap().files.len()
    }

    /// Number of files removed by a dropped sink or `discard`.
    pub fn deleted_count(&self) -> usize {
        self.state.lock().unwrap().deleted.len()
    }

    /// Removes a staged file without counting it as deleted: it was moved
    /// into the blob store.
    pub fn take(&self, media: &StagedMedia) -> Option<Vec<u8>> {
        self.state.lock().unwrap().files.remove(&media.address)
    }

    fn next_address(state: &mut StagingState) -> String {
        state.next_id += 1;
        format!("staged-{}", state.next_id)
    }
}

struct FakeSink {
    state: Arc<Mutex<StagingState>>,
    address: String,
    kept: bool,
}

#[async_trait::async_trait]
impl StagedSink for FakeSink {
    async fn append(&mut self, chunk: &[u8]) -> Result<(), StagingError> {
        let mut state = self.state.lock().unwrap();
        let file = state
            .files
            .get_mut(&self.address)
            .ok_or_else(|| StagingError::Io("staged file vanished".to_owned()))?;
        file.extend_from_slice(chunk);
        Ok(())
    }

    async fn keep(mut self: Box<Self>) -> Result<StagedMedia, StagingError> {
        self.kept = true;
        let size_bytes = self.state.lock().unwrap().files[&self.address].len() as u64;
        Ok(StagedMedia {
            address: self.address.clone(),
            size_bytes,
        })
    }
}

impl Drop for FakeSink {
    fn drop(&mut self) {
        if self.kept {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if state.files.remove(&self.address).is_some() {
            state.deleted.push(self.address.clone());
        }
    }
}

#[async_trait::async_trait]
impl UploadStaging for FakeUploadStaging {
    async fn free_bytes(&self) -> Result<u64, StagingError> {
        self.free_bytes
            .lock()
            .unwrap()
            .ok_or(StagingError::Unsupported)
    }

    async fn open(&self) -> Result<Box<dyn StagedSink>, StagingError> {
        let mut state = self.state.lock().unwrap();
        let address = Self::next_address(&mut state);
        state.files.insert(address.clone(), Vec::new());
        Ok(Box::new(FakeSink {
            state: Arc::clone(&self.state),
            address,
            kept: false,
        }))
    }

    async fn read_prefix(
        &self,
        media: &StagedMedia,
        max_bytes: usize,
    ) -> Result<Vec<u8>, StagingError> {
        let state = self.state.lock().unwrap();
        let file = state
            .files
            .get(&media.address)
            .ok_or_else(|| StagingError::Io("no such staged file".to_owned()))?;
        Ok(file[..file.len().min(max_bytes)].to_vec())
    }

    async fn discard(&self, media: &StagedMedia) -> Result<(), StagingError> {
        let mut state = self.state.lock().unwrap();
        if state.files.remove(&media.address).is_some() {
            state.deleted.push(media.address.clone());
        }
        Ok(())
    }
}
