use super::list_library_query::ListLibraryQuery;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct ListLibraryQueryValidator;

impl ListLibraryQueryValidator {
    /// Any search text is acceptable; a blank one just means "no filter".
    pub fn validate(&self, _query: &ListLibraryQuery) -> Result<(), ApplicationError> {
        Ok(())
    }

    /// The search text to filter by, or `None` when it is absent or blank.
    pub fn non_blank_search<'a>(&self, query: &'a ListLibraryQuery) -> Option<&'a str> {
        query
            .query
            .as_deref()
            .filter(|text| !text.trim().is_empty())
    }
}

#[cfg(test)]
#[path = "list_library_query_validator.tests.rs"]
mod tests;
