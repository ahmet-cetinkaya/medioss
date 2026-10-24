/// Executes one command (a mutation). Each use case implements this once, for
/// its own `<Name>Command`; `Output` and `Error` are that use case's result
/// and failure types.
#[async_trait::async_trait]
pub trait CommandHandler<C>: Send + Sync {
    type Output;
    type Error;

    async fn handle(&self, command: C) -> Result<Self::Output, Self::Error>;
}
