use crate::actor::role::Role;
use crate::actor::user_id::UserId;

/// Who is acting. Fields are private so M1 can move construction behind a
/// session-verification path without touching read sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Actor {
    user_id: UserId,
    role: Role,
}

impl Actor {
    pub fn new(user_id: UserId, role: Role) -> Self {
        Self { user_id, role }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn role(&self) -> Role {
        self.role
    }
}
