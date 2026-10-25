use kernel::UserId;

#[derive(Debug)]
pub struct ExportOwnDataQuery {
    pub user_id: UserId,
}
