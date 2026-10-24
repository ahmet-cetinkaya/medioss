use kernel::{Actor, UserId};

/// Deletes a user as an admin. Deleting the last active admin is refused.
/// Callers must run the guard (and their purge step) before this; the
/// guards run again here so the command stays safe when called directly.
#[derive(Debug, Clone)]
pub struct DeleteUserAsAdmin {
    pub actor: Actor,
    pub target: UserId,
}
