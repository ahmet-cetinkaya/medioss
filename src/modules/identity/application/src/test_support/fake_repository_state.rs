use std::collections::HashMap;

use identity_domain::User;

#[derive(Clone, Default)]
pub(super) struct FakeRepositoryState {
    pub(super) users: HashMap<String, User>,
    pub(super) failed_attempts: HashMap<String, u64>,
    pub(super) next_user_id: i64,
    pub(super) count_calls: usize,
    pub(super) initial_admin_insert_calls: usize,
    pub(super) credential_change_calls: usize,
    pub(super) should_fail_credential_change: bool,
}
