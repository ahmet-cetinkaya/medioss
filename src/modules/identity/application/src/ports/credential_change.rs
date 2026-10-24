use identity_domain::PasswordHash;
use kernel::UserId;

pub struct CredentialChange {
    user_id: UserId,
    password_hash: PasswordHash,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CredentialChange {
    pub fn permanent(user_id: UserId, password_hash: PasswordHash) -> Self {
        Self {
            user_id,
            password_hash,
            expires_at: None,
        }
    }

    pub fn temporary(
        user_id: UserId,
        password_hash: PasswordHash,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            user_id,
            password_hash,
            expires_at: Some(expires_at),
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.expires_at
    }
}

#[cfg(test)]
#[path = "credential_change.tests.rs"]
mod tests;
