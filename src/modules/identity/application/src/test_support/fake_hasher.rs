use std::sync::Mutex;

use async_trait::async_trait;
use identity_domain::{PasswordHash, RawPassword};

use crate::ports::{PasswordHasher, PasswordHasherError};

pub(crate) struct FakeHasher {
    verified_hashes: Mutex<Vec<PasswordHash>>,
    hash_calls: Mutex<usize>,
}

impl FakeHasher {
    pub(crate) fn new() -> Self {
        Self {
            verified_hashes: Mutex::new(Vec::new()),
            hash_calls: Mutex::new(0),
        }
    }

    pub(crate) fn hash_for(password: &RawPassword) -> PasswordHash {
        PasswordHash::from_stored(format!("fake-hash:{}", password.as_str()))
    }

    pub(crate) fn hash_calls(&self) -> usize {
        *self.hash_calls.lock().unwrap()
    }

    pub(crate) fn verify_calls(&self) -> usize {
        self.verified_hashes.lock().unwrap().len()
    }

    pub(crate) fn last_verified_hash(&self) -> Option<PasswordHash> {
        self.verified_hashes.lock().unwrap().last().cloned()
    }
}

#[async_trait]
impl PasswordHasher for FakeHasher {
    async fn hash(&self, password: &RawPassword) -> Result<PasswordHash, PasswordHasherError> {
        let mut calls = self.hash_calls.lock().unwrap();
        *calls += 1;
        Ok(Self::hash_for(password))
    }

    async fn verify(
        &self,
        password: &str,
        password_hash: &PasswordHash,
    ) -> Result<bool, PasswordHasherError> {
        self.verified_hashes
            .lock()
            .unwrap()
            .push(password_hash.clone());
        Ok(PasswordHash::from_stored(format!("fake-hash:{password}")) == *password_hash)
    }
}
