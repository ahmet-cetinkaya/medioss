//! Secret used to prove operator possession during first-run setup.

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct SessionSecret(String);

impl SessionSecret {
    pub(super) fn new(value: String) -> Self {
        Self(value)
    }

    /// Exposes the secret only to consumers that must perform possession checks.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SessionSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionSecret([REDACTED])")
    }
}
