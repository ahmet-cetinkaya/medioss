use std::sync::{Arc, Mutex};

use crate::ports::{BlobStore, StoredBlob};

/// In-memory blob store recording every put and delete.
#[derive(Clone, Default)]
pub struct FakeBlobStore {
    state: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    counter: Arc<Mutex<u64>>,
    pub failures: Arc<Mutex<bool>>,
}

impl FakeBlobStore {
    /// A store that starts empty and succeeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored blobs.
    pub fn len(&self) -> usize {
        self.state.lock().unwrap().len()
    }

    /// Whether the store holds no blobs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether a blob exists at this exact key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.state.lock().unwrap().contains_key(key)
    }
}

#[async_trait::async_trait]
impl BlobStore for FakeBlobStore {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<StoredBlob, String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        let key = format!(
            "{folder}/{stem}-{}.{}",
            {
                let mut counter = self.counter.lock().unwrap();
                *counter += 1;
                *counter
            },
            ext
        );
        self.state
            .lock()
            .unwrap()
            .insert(key.clone(), bytes.to_vec());
        Ok(StoredBlob {
            storage_key: key,
            size_bytes: bytes.len() as i64,
        })
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        self.state.lock().unwrap().remove(key);
        Ok(())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        self.state
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| "no blob".to_owned())
    }

    async fn relocate(
        &self,
        key: &str,
        folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<String, String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        let bytes = self
            .state
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| "no blob".to_owned())?;
        let new_key = format!("{folder}/{stem}-moved.{ext}");
        let mut state = self.state.lock().unwrap();
        state.remove(key);
        state.insert(new_key.clone(), bytes);
        Ok(new_key)
    }

    async fn delete_folder(&self, folder: &str) -> Result<(), String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        self.state
            .lock()
            .unwrap()
            .retain(|key, _| !key.starts_with(&format!("{folder}/")));
        Ok(())
    }
}
