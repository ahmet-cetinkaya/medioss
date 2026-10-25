use super::export_own_data_query::ExportOwnDataQuery;

#[derive(Debug, Default, Clone, Copy)]
pub struct ExportOwnDataQueryValidator;

impl ExportOwnDataQueryValidator {
    pub fn validate(&self, _query: &ExportOwnDataQuery) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "export_own_data_query_validator.tests.rs"]
mod tests;
