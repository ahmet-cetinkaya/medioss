use kernel::UserId;

#[derive(Debug, Clone)]
pub struct UnfavoriteMeditationCommand {
    pub user_id: UserId,
    pub meditation_id: i64,
}
