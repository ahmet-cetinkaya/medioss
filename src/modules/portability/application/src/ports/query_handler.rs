/// Answers one read-only query. Each use case implements this once, for its
/// own `<Name>Query`; `Output` and `Error` are that use case's result and
/// failure types.
#[async_trait::async_trait]
pub trait QueryHandler<Q>: Send + Sync {
    type Output;
    type Error;

    async fn handle(&self, query: Q) -> Result<Self::Output, Self::Error>;
}
