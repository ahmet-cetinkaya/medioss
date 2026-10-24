use kernel::UserId;
#[derive(Debug, Clone)]
pub struct ListRecentProgramsQuery {
    pub user_id: UserId,
    pub limit: Option<usize>,
}
