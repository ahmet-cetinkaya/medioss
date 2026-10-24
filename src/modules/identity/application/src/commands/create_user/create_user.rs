use identity_domain::{Email, RawPassword};
use kernel::Actor;

/// Registers a member with temporary credentials. Admin-only; the member
/// must change the password at first login.
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub actor: Actor,
    pub email: Email,
    pub temporary_password: RawPassword,
}
