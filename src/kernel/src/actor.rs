use crate::role::Role;
use crate::user_id::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub user_id: UserId,
    pub role: Role,
}
