use kernel::UserId;
use soundscape_domain::MixItem;

#[derive(Debug, Clone)]
pub struct CreateMixCommand {
    pub user_id: UserId,
    pub name: String,
    pub items: Vec<MixItem>,
}
