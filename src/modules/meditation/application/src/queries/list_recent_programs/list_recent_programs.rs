use kernel::UserId;
#[derive(Debug, Clone)]
pub struct ListRecentPrograms {
    pub user_id: UserId,
    pub limit: Option<usize>,
}
