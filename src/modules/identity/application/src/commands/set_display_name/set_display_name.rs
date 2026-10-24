use identity_domain::DisplayName;
use kernel::UserId;

/// Sets or clears the caller's display name. `None` clears an existing
/// name; the avatar is preserved.
#[derive(Debug, Clone)]
pub struct SetDisplayName {
    pub user_id: UserId,
    pub display_name: Option<DisplayName>,
}
