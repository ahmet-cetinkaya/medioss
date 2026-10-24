use crate::ApplicationError;

/// Answers one read-only query. Each use case implements this once, for its
/// own `<Name>Query`; `Output` is the display-shaped result.
#[async_trait::async_trait]
pub trait QueryHandler<Q>: Send + Sync {
    type Output;

    async fn handle(&self, query: Q) -> Result<Self::Output, ApplicationError>;
}
