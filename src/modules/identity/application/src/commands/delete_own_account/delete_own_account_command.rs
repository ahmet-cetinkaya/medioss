use kernel::Actor;

/// Deletes the caller's own account. Refused when the caller is the last
/// active admin. Callers must run the guard (and their purge step) before
/// this; the guard runs again here so the command stays safe when called
/// directly.
#[derive(Debug, Clone)]
pub struct DeleteOwnAccountCommand {
    pub actor: Actor,
}
