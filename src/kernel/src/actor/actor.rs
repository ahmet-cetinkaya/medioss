use crate::actor::role::Role;
use crate::actor::user_id::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub user_id: UserId,
    pub role: Role,
}
