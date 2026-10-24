use chrono::{DateTime, Utc};
use identity_domain::{Email, PasswordHash};

pub struct MemberRegistration {
    email: Email,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
}

impl MemberRegistration {
    pub fn new(email: Email, password_hash: PasswordHash, created_at: DateTime<Utc>) -> Self {
        Self {
            email,
            password_hash,
            created_at,
        }
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
#[path = "member_registration.tests.rs"]
mod tests;
