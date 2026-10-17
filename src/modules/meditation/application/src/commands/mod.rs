//! Commands: state-changing meditation use-cases. One file per command
//! (identity-application convention); shared validation helpers live here.
mod add_member;
mod create_category;
mod create_note;
mod create_program;
mod delete_category;
mod delete_meditation;
mod delete_note;
mod delete_program;
mod record_completion;
mod record_position;
mod remove_member;
mod rename_category;
mod set_completed;
mod set_program_category;
mod update_meditation_description;
mod update_note;
mod update_program;
mod update_program_order;
mod upload_meditation;

pub use add_member::add_member;
pub use create_category::create_category;
pub use create_note::create_note;
pub use create_program::create_program;
pub use delete_category::delete_category;
pub use delete_meditation::delete_meditation;
pub use delete_note::delete_note;
pub use delete_program::delete_program;
pub use record_completion::record_completion;
pub use record_position::record_position;
pub use remove_member::remove_member;
pub use rename_category::rename_category;
pub use set_completed::set_completed;
pub use set_program_category::set_program_category;
pub use update_meditation_description::update_meditation_description;
pub use update_note::update_note;
pub use update_program::update_program;
pub use update_program_order::update_program_order;
pub use upload_meditation::upload_meditation;

use crate::ports::MeditationRepository;
use crate::ApplicationError;

/// Upload size cap: 100MB applies to the original bytes (spec FR-001/002).
pub const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;

/// Cover size cap: 5MB (spec FR-026).
pub const MAX_COVER_BYTES: usize = 5 * 1024 * 1024;

/// Name length cap, mirroring the domain invariant.
pub const MAX_NAME_LENGTH: usize = 200;

pub(crate) async fn ensure_meditation_exists<R>(
    repository: &R,
    meditation_id: i64,
) -> Result<(), ApplicationError>
where
    R: MeditationRepository + ?Sized,
{
    let found = repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    if found.is_some() {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}

pub(crate) fn validate_name(name: &str) -> Result<(), ApplicationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(ApplicationError::InvalidInput(
            "name must be 1-200 characters",
        ));
    }
    Ok(())
}

/// Normalizes an optional description (blank clears it) and enforces the
/// domain length limit.
pub(crate) fn validate_description(
    description: Option<&str>,
) -> Result<Option<String>, ApplicationError> {
    let text = description.map(str::trim).filter(|text| !text.is_empty());
    if text.is_some_and(|text| text.chars().count() > meditation_domain::MAX_DESCRIPTION_LENGTH) {
        return Err(ApplicationError::InvalidDescription);
    }
    Ok(text.map(str::to_owned))
}

#[cfg(test)]
#[path = "commands_tests/mod.rs"]
mod tests;
