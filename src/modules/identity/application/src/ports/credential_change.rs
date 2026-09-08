use identity_domain::PasswordHash;
use kernel::UserId;

pub struct CredentialChange {
    user_id: UserId,
    password_hash: PasswordHash,
}

impl CredentialChange {
    pub fn new(user_id: UserId, password_hash: PasswordHash) -> Self {
        Self {
            user_id,
            password_hash,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }
}
