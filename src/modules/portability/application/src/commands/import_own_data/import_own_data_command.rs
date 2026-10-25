use crate::models::export_document::ExportDocument;
use kernel::UserId;

#[derive(Debug)]
pub struct ImportOwnDataCommand {
    pub user_id: UserId,
    pub document: ExportDocument,
}

#[derive(Debug)]
pub enum ImportFailure {
    Validation(String),
    Internal(String),
}

impl From<String> for ImportFailure {
    fn from(e: String) -> Self {
        ImportFailure::Internal(e)
    }
}

impl std::fmt::Display for ImportFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportFailure::Validation(m) => write!(f, "{m}"),
            ImportFailure::Internal(m) => write!(f, "{m}"),
        }
    }
}

#[cfg(test)]
#[path = "import_own_data_command.tests.rs"]
mod tests;
