use kernel::UserId;

#[derive(Debug, Clone)]
pub struct ListFavoriteIdsQuery {
    pub user_id: UserId,
}
