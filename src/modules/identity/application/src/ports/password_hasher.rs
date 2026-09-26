use async_trait::async_trait;
use identity_domain::{PasswordHash, RawPassword};

use super::PasswordHasherError;

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &RawPassword) -> Result<PasswordHash, PasswordHasherError>;

    async fn verify(
        &self,
        password: &str,
        password_hash: &PasswordHash,
    ) -> Result<bool, PasswordHasherError>;
}
