use identity_domain::{Email, RawPassword};

/// Registers the first admin. Refused once any user exists; the atomic
/// insert decides, so a concurrent setup cannot create two admins.
#[derive(Debug, Clone)]
pub struct SetupAdmin {
    pub email: Email,
    pub password: RawPassword,
}
