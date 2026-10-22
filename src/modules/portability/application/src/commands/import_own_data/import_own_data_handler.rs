use super::import_own_data::{ImportFailure, ImportOwnData};
use crate::models::import_report::ImportReport;

#[async_trait::async_trait]
pub trait ImportOwnDataHandler: Send + Sync {
    async fn handle(&self, command: ImportOwnData) -> Result<ImportReport, ImportFailure>;
}
