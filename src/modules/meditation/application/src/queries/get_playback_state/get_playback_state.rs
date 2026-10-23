use kernel::UserId;
#[derive(Debug, Clone)]
pub struct GetPlaybackState {
    pub user_id: UserId,
    pub meditation_id: i64,
}
