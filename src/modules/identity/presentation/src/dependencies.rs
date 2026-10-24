use identity_application::commands::authenticate::AuthenticateHandler;
use identity_application::commands::change_password::ChangePasswordHandler;
use identity_application::commands::clear_avatar::ClearAvatarHandler;
use identity_application::commands::clear_login_throttle::ClearLoginThrottleHandler;
use identity_application::commands::create_user::CreateUserHandler;
use identity_application::commands::delete_own_account::DeleteOwnAccountHandler;
use identity_application::commands::delete_user_as_admin::DeleteUserAsAdminHandler;
use identity_application::commands::read_avatar::ReadAvatarHandler;
use identity_application::commands::replace_avatar::ReplaceAvatarHandler;
use identity_application::commands::set_display_name::SetDisplayNameHandler;
use identity_application::commands::set_user_status::SetUserStatusHandler;
use identity_application::commands::setup_admin::SetupAdminHandler;
use identity_application::queries::count_users::CountUsersHandler;
use identity_application::queries::find_user_by_id::FindUserByIdHandler;
use identity_application::queries::list_users::ListUsersHandler;
use identity_application::queries::setup_status::IsSetupRequiredHandler;
use identity_domain::PasswordHash;
use std::sync::Arc;

#[derive(Clone)]
pub struct IdentityDependencies {
    pub authenticate: Arc<dyn AuthenticateHandler>,
    pub create_user: Arc<dyn CreateUserHandler>,
    pub change_password: Arc<dyn ChangePasswordHandler>,
    pub delete_own_account: Arc<dyn DeleteOwnAccountHandler>,
    pub delete_user_as_admin: Arc<dyn DeleteUserAsAdminHandler>,
    pub set_display_name: Arc<dyn SetDisplayNameHandler>,
    pub set_user_status: Arc<dyn SetUserStatusHandler>,
    pub setup_admin: Arc<dyn SetupAdminHandler>,
    pub clear_login_throttle: Arc<dyn ClearLoginThrottleHandler>,
    pub replace_avatar: Arc<dyn ReplaceAvatarHandler>,
    pub clear_avatar: Arc<dyn ClearAvatarHandler>,
    pub read_avatar: Arc<dyn ReadAvatarHandler>,
    pub count_users: Arc<dyn CountUsersHandler>,
    pub find_user_by_id: Arc<dyn FindUserByIdHandler>,
    pub list_users: Arc<dyn ListUsersHandler>,
    pub setup_status: Arc<dyn IsSetupRequiredHandler>,
    pub dummy_password_hash: PasswordHash,
    pub session_secret: Arc<String>,
    pub medioss_origin: Arc<String>,
    pub deletion: Arc<dyn crate::users_lifecycle::UserDeletionListener>,
    pub avatar_store: Arc<dyn identity_application::ports::AvatarStore>,
}

pub type SharedDependencies = Arc<IdentityDependencies>;
