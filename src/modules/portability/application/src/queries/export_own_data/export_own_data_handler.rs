use super::export_own_data::ExportOwnData;
use crate::models::export_document::ExportDocument;

#[async_trait::async_trait]
pub trait ExportOwnDataHandler: Send + Sync {
    async fn handle(&self, command: ExportOwnData) -> Result<ExportDocument, String>;
}
