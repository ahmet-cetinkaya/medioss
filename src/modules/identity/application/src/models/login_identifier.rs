/// Canonical form of a login identifier: throttle rows and e-mail lookups key
/// on it, so every use case must agree on it.
pub fn normalize_login_identifier(raw: &str) -> String {
    raw.trim().to_lowercase()
}

#[cfg(test)]
#[path = "login_identifier.tests.rs"]
mod tests;
