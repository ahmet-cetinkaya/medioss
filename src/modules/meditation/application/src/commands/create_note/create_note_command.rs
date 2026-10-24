use kernel::UserId;
#[derive(Debug, Clone)]
pub struct CreateNoteCommand {
    pub user_id: UserId,
    pub meditation_id: i64,
    pub text: String,
}
