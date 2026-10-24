mod delete_user_as_admin;
mod delete_user_as_admin_handler;
mod delete_user_as_admin_service;
pub(crate) mod last_active_admin_guard;

pub use delete_user_as_admin::DeleteUserAsAdmin;
pub use delete_user_as_admin_handler::DeleteUserAsAdminHandler;
pub use delete_user_as_admin_service::DeleteUserAsAdminService;
