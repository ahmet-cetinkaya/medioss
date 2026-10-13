use std::fmt;

use super::IdentityError;

#[derive(Clone, PartialEq, Eq)]
pub enum AvatarRef {
    Webp,
    Png,
    Stored {
        key: String,
        extension: AvatarExtension,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AvatarExtension {
    Webp,
    Png,
}

impl AvatarRef {
    pub fn from_extension(value: &str) -> Result<Self, IdentityError> {
        Ok(match AvatarExtension::parse(value)? {
            AvatarExtension::Webp => Self::Webp,
            AvatarExtension::Png => Self::Png,
        })
    }

    pub fn new(key: String, extension: &str) -> Result<Self, IdentityError> {
        let extension = AvatarExtension::parse(extension)?;
        if !is_minted_key(&key, extension) {
            return Err(IdentityError::InvalidProfile);
        }

        Ok(Self::Stored { key, extension })
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Stored { key, .. } => key,
            Self::Webp => "avatar.webp",
            Self::Png => "avatar.png",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Png => "png",
            Self::Stored { extension, .. } => extension.as_str(),
        }
    }
}

const KEY_PREFIX: &str = "avatar-";
const KEY_RANDOM_HEX_LENGTH: usize = 16;

/// Accepts only the shape the avatar store mints:
/// `avatar-<16 lowercase hex>.<extension>`.
fn is_minted_key(key: &str, extension: AvatarExtension) -> bool {
    let Some(rest) = key.strip_prefix(KEY_PREFIX) else {
        return false;
    };
    let Some((random, suffix)) = rest.split_once('.') else {
        return false;
    };
    random.len() == KEY_RANDOM_HEX_LENGTH
        && random
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && suffix == extension.as_str()
}

impl AvatarExtension {
    fn parse(value: &str) -> Result<Self, IdentityError> {
        match value {
            "webp" => Ok(Self::Webp),
            "png" => Ok(Self::Png),
            _ => Err(IdentityError::InvalidProfile),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Png => "png",
        }
    }
}

impl fmt::Debug for AvatarRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AvatarRef")
            .field("key", &self.key())
            .field("extension", &self.extension())
            .finish()
    }
}

#[cfg(test)]
#[path = "avatar_ref.tests.rs"]
mod tests;
