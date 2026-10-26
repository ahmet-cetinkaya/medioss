use kernel::UserId;

#[derive(Debug, Clone)]
pub struct ListFavoriteMeditationsQuery {
    pub user_id: UserId,
}
