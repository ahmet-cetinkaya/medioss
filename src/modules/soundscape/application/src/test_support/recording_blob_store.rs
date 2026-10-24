use std::sync::Mutex;

use async_trait::async_trait;

use crate::ports::media::{BlobReader, BlobStore};

/// Records every put/delete so tests can assert on blob side effects.
#[derive(Default)]
pub struct RecordingBlobStore {
    pub puts: Mutex<Vec<String>>,
    pub deletes: Mutex<Vec<String>>,
    pub fail_put: bool,
    pub fail_delete: bool,
}

impl RecordingBlobStore {
    pub fn put_keys(&self) -> Vec<String> {
        self.puts.lock().unwrap().clone()
    }

    pub fn deleted_keys(&self) -> Vec<String> {
        self.deletes.lock().unwrap().clone()
    }
}

#[async_trait]
impl BlobStore for RecordingBlobStore {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        _bytes: &[u8],
    ) -> Result<String, String> {
        if self.fail_put {
            return Err("put failed".into());
        }
        let mut puts = self.puts.lock().unwrap();
        let key = format!("{folder}/{stem}-{}.{ext}", puts.len());
        puts.push(key.clone());
        Ok(key)
    }

    async fn open_range(
        &self,
        _: &str,
        _: u64,
        _: u64,
    ) -> Result<(BlobReader, String, u64), String> {
        Err("not supported by the recording store".into())
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        self.deletes.lock().unwrap().push(key.into());
        if self.fail_delete {
            return Err("delete failed".into());
        }
        Ok(())
    }
}
