use identity_application::commands::{
    authenticate::AuthenticateCommand, change_password::ChangePasswordCommand,
    clear_avatar::ClearAvatarCommand, clear_login_throttle::ClearLoginThrottleCommand,
    create_user::CreateUserCommand, delete_own_account::DeleteOwnAccountCommand,
    delete_user_as_admin::DeleteUserAsAdminCommand, read_avatar::ReadAvatarCommand,
    replace_avatar::ReplaceAvatarCommand, set_display_name::SetDisplayNameCommand,
    set_user_status::SetUserStatusCommand, setup_admin::SetupAdminCommand,
};
use identity_application::ports::{
    CommandHandler, OwnAccountDeletionGuard, QueryHandler, UserDeletionGuard,
};
use identity_application::queries::{
    count_users::CountUsersQuery, find_user_by_id::FindUserByIdQuery, list_users::ListUsersQuery,
    setup_status::SetupStatusQuery,
};
use identity_domain::{PasswordHash, User};
use std::sync::Arc;

#[derive(Clone)]
pub struct IdentityDependencies {
    pub authenticate: Arc<dyn CommandHandler<AuthenticateCommand, Output = User>>,
    pub create_user: Arc<dyn CommandHandler<CreateUserCommand, Output = User>>,
    pub change_password: Arc<dyn CommandHandler<ChangePasswordCommand, Output = User>>,
    pub delete_own_account: Arc<dyn CommandHandler<DeleteOwnAccountCommand, Output = ()>>,
    pub delete_user_as_admin: Arc<dyn CommandHandler<DeleteUserAsAdminCommand, Output = ()>>,
    pub set_display_name: Arc<dyn CommandHandler<SetDisplayNameCommand, Output = User>>,
    pub set_user_status: Arc<dyn CommandHandler<SetUserStatusCommand, Output = ()>>,
    pub setup_admin: Arc<dyn CommandHandler<SetupAdminCommand, Output = User>>,
    pub clear_login_throttle: Arc<dyn CommandHandler<ClearLoginThrottleCommand, Output = ()>>,
    pub replace_avatar: Arc<dyn CommandHandler<ReplaceAvatarCommand, Output = User>>,
    pub clear_avatar: Arc<dyn CommandHandler<ClearAvatarCommand, Output = ()>>,
    pub read_avatar:
        Arc<dyn CommandHandler<ReadAvatarCommand, Output = (Vec<u8>, identity_domain::AvatarRef)>>,
    pub count_users: Arc<dyn QueryHandler<CountUsersQuery, Output = u64>>,
    pub find_user_by_id: Arc<dyn QueryHandler<FindUserByIdQuery, Output = Option<User>>>,
    pub list_users: Arc<dyn QueryHandler<ListUsersQuery, Output = Vec<User>>>,
    pub setup_status: Arc<dyn QueryHandler<SetupStatusQuery, Output = bool>>,
    pub own_account_deletion_guard: Arc<dyn OwnAccountDeletionGuard>,
    pub user_deletion_guard: Arc<dyn UserDeletionGuard>,
    pub dummy_password_hash: PasswordHash,
    pub session_secret: Arc<String>,
    pub medioss_origin: Arc<String>,
    pub deletion: Arc<dyn crate::users_lifecycle::UserDeletionListener>,
    pub avatar_store: Arc<dyn identity_application::ports::AvatarStore>,
}

pub type SharedDependencies = Arc<IdentityDependencies>;
