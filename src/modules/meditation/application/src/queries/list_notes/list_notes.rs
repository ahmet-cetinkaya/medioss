use kernel::UserId;
#[derive(Debug, Clone)]
pub struct ListNotes {
    pub user_id: UserId,
    pub meditation_id: i64,
}
