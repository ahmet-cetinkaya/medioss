use kernel::{Actor, UserId};

/// Sets the disabled flag of a user. Admin-only. Disabling oneself is
/// always refused; disabling the last active admin is refused to keep the
/// instance administrable.
#[derive(Debug, Clone)]
pub struct SetUserStatusCommand {
    pub actor: Actor,
    pub target: UserId,
    pub disabled: bool,
}
