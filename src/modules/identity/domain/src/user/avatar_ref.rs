use std::fmt;

use super::IdentityError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AvatarRef {
    Webp,
    Png,
}

impl AvatarRef {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Png => "png",
        }
    }

    pub fn from_extension(value: &str) -> Result<Self, IdentityError> {
        match value {
            "webp" => Ok(Self::Webp),
            "png" => Ok(Self::Png),
            _ => Err(IdentityError::InvalidProfile),
        }
    }
}

impl fmt::Debug for AvatarRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("AvatarRef({})", self.extension()))
    }
}

#[cfg(test)]
#[path = "avatar_ref.tests.rs"]
mod tests;
