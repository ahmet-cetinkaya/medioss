use super::export_mood_records_query::ExportMoodRecordsQuery;

#[derive(Debug, Default, Clone, Copy)]
pub struct ExportMoodRecordsQueryValidator;

impl ExportMoodRecordsQueryValidator {
    pub fn validate(&self, _query: &ExportMoodRecordsQuery) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "export_mood_records_query_validator.tests.rs"]
mod tests;
