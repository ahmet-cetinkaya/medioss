use std::fmt;

use chrono::{DateTime, TimeDelta, Utc};
use kernel::{Actor, Role, UserId};

use super::{AvatarRef, DisplayName, Email, IdentityError, PasswordHash};

const TEMPORARY_PASSWORD_LIFETIME: TimeDelta = TimeDelta::hours(24);

#[derive(Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
    role: Role,
    created_at: DateTime<Utc>,
    auth_version: u64,
    must_change_password: bool,
    password_expires_at: Option<DateTime<Utc>>,
    disabled: bool,
    display_name: Option<DisplayName>,
    avatar: Option<AvatarRef>,
}

impl User {
    pub fn new_permanent(
        id: UserId,
        email: Email,
        password_hash: PasswordHash,
        role: Role,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            role,
            created_at,
            auth_version: 0,
            must_change_password: false,
            password_expires_at: None,
            disabled: false,
            display_name: None,
            avatar: None,
        }
    }

    pub fn new_temporary(
        id: UserId,
        email: Email,
        password_hash: PasswordHash,
        role: Role,
        created_at: DateTime<Utc>,
    ) -> Result<Self, IdentityError> {
        let password_expires_at = temporary_password_expiry(created_at)?;

        Ok(Self {
            id,
            email,
            password_hash,
            role,
            created_at,
            auth_version: 0,
            must_change_password: true,
            password_expires_at: Some(password_expires_at),
            disabled: false,
            display_name: None,
            avatar: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: UserId,
        email: Email,
        password_hash: PasswordHash,
        role: Role,
        created_at: DateTime<Utc>,
        auth_version: u64,
        must_change_password: bool,
        password_expires_at: Option<DateTime<Utc>>,
        disabled: bool,
        display_name: Option<DisplayName>,
        avatar: Option<AvatarRef>,
    ) -> Result<Self, IdentityError> {
        if must_change_password != password_expires_at.is_some() {
            return Err(IdentityError::InvalidCredentialState);
        }

        Ok(Self {
            id,
            email,
            password_hash,
            role,
            created_at,
            auth_version,
            must_change_password,
            password_expires_at,
            disabled,
            display_name,
            avatar,
        })
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn auth_version(&self) -> u64 {
        self.auth_version
    }

    pub fn must_change_password(&self) -> bool {
        self.must_change_password
    }

    pub fn password_expires_at(&self) -> Option<DateTime<Utc>> {
        self.password_expires_at
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn display_name(&self) -> Option<&DisplayName> {
        self.display_name.as_ref()
    }

    pub fn avatar(&self) -> Option<AvatarRef> {
        self.avatar
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    pub fn with_display_name(self, display_name: Option<DisplayName>) -> Self {
        Self {
            display_name,
            ..self
        }
    }

    pub fn with_avatar(self, avatar: Option<AvatarRef>) -> Self {
        Self { avatar, ..self }
    }

    pub fn actor(&self) -> Actor {
        Actor::new(self.id, self.role)
    }

    pub fn ensure_password_is_active_at(&self, now: DateTime<Utc>) -> Result<(), IdentityError> {
        if self
            .password_expires_at
            .is_some_and(|expires_at| now >= expires_at)
        {
            return Err(IdentityError::InvalidCredentials);
        }

        Ok(())
    }

    pub fn replace_with_permanent_password(
        self,
        password_hash: PasswordHash,
    ) -> Result<Self, IdentityError> {
        Ok(Self {
            password_hash,
            auth_version: increment_auth_version(self.auth_version)?,
            must_change_password: false,
            password_expires_at: None,
            ..self
        })
    }

    pub fn replace_with_temporary_password(
        self,
        password_hash: PasswordHash,
        changed_at: DateTime<Utc>,
    ) -> Result<Self, IdentityError> {
        Ok(Self {
            password_hash,
            auth_version: increment_auth_version(self.auth_version)?,
            must_change_password: true,
            password_expires_at: Some(temporary_password_expiry(changed_at)?),
            ..self
        })
    }
}

impl fmt::Debug for User {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("User")
            .field("id", &self.id)
            .field("email", &"[REDACTED]")
            .field("password_hash", &"[REDACTED]")
            .field("role", &self.role)
            .field("created_at", &self.created_at)
            .field("auth_version", &self.auth_version)
            .field("must_change_password", &self.must_change_password)
            .field("password_expires_at", &self.password_expires_at)
            .finish()
    }
}

fn temporary_password_expiry(changed_at: DateTime<Utc>) -> Result<DateTime<Utc>, IdentityError> {
    changed_at
        .checked_add_signed(TEMPORARY_PASSWORD_LIFETIME)
        .ok_or(IdentityError::InvalidCredentialState)
}

fn increment_auth_version(auth_version: u64) -> Result<u64, IdentityError> {
    auth_version
        .checked_add(1)
        .ok_or(IdentityError::AuthVersionOverflow)
}

#[cfg(test)]
#[path = "user.tests.rs"]
mod tests;
