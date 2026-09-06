#[derive(Debug, Clone, PartialEq)]
pub struct UserId(pub i64);

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub user_id: UserId,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Member,
}
