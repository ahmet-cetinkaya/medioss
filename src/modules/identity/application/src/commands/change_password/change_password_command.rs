use identity_domain::RawPassword;
use kernel::UserId;

/// Replaces the actor's password with a permanent credential change and
/// bumps the auth version (invalidating other sessions).
#[derive(Debug, Clone)]
pub struct ChangePasswordCommand {
    pub user_id: UserId,
    pub new_password: RawPassword,
}
