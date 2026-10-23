use super::get_playback_state::GetPlaybackState;
use crate::ports::PlaybackState;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetPlaybackStateHandler: Send + Sync {
    async fn handle(&self, query: GetPlaybackState) -> Result<PlaybackState, ApplicationError>;
}
