use crate::ApplicationError;

/// Executes one command (a mutation). Each use case implements this once, for
/// its own `<Name>Command`; `Output` is what the caller gets back.
#[async_trait::async_trait]
pub trait CommandHandler<C>: Send + Sync {
    type Output;

    async fn handle(&self, command: C) -> Result<Self::Output, ApplicationError>;
}
