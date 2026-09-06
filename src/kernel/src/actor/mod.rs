//! Who is acting: the identity and permission types every module reasons about.

// The file name mirrors the domain topic; splitting it out keeps one type per file.
#[allow(clippy::module_inception)]
pub mod actor;
pub mod role;
pub mod user_id;

pub use actor::Actor;
pub use role::Role;
pub use user_id::UserId;
