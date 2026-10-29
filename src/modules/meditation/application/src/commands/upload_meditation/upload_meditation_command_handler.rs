use std::sync::Arc;

use super::upload_meditation_command::UploadMeditationCommand;
use super::upload_meditation_command_validator::UploadMeditationCommandValidator;
use crate::ports::{
    AudioValidator, BlobStore, Clock, CommandHandler, MeditationRepository, MembershipRepository,
    OpusConverter, ProgramRepository, StagedMedia, UploadStaging,
};
use crate::ports::{ConvertError, StoredAudio};
use crate::ApplicationError;
use meditation_domain::Meditation;

/// How much of the staged file the format sniffer gets to see.
const SNIFF_PREFIX_BYTES: usize = 64 * 1024;

pub struct UploadMeditationCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
    validator: Arc<dyn AudioValidator>,
    converter: Arc<dyn OpusConverter>,
    store: Arc<dyn BlobStore>,
    staging: Arc<dyn UploadStaging>,
    clock: Arc<dyn Clock>,
    input_validator: UploadMeditationCommandValidator,
}

impl UploadMeditationCommandHandler {
    #[allow(clippy::too_many_arguments)] // every port plus the injected size cap
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
        validator: Arc<dyn AudioValidator>,
        converter: Arc<dyn OpusConverter>,
        store: Arc<dyn BlobStore>,
        staging: Arc<dyn UploadStaging>,
        clock: Arc<dyn Clock>,
        max_upload_bytes: usize,
    ) -> Self {
        Self {
            meditations,
            memberships,
            programs,
            validator,
            converter,
            store,
            staging,
            clock,
            input_validator: UploadMeditationCommandValidator::new(max_upload_bytes),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UploadMeditationCommand> for UploadMeditationCommandHandler {
    type Output = Meditation;

    async fn handle(
        &self,
        command: UploadMeditationCommand,
    ) -> Result<Meditation, ApplicationError> {
        let staged = command.media.clone();
        let result = self.upload(command).await;
        if result.is_err() {
            // Idempotent: a file already moved or converted away is absent.
            let _ = self.staging.discard(&staged).await;
        }
        result
    }
}

impl UploadMeditationCommandHandler {
    async fn upload(
        &self,
        command: UploadMeditationCommand,
    ) -> Result<Meditation, ApplicationError> {
        self.input_validator.validate(&command)?;

        // Garbage bytes, oversized payloads, and conversion failures leave
        // nothing stored (spec FR-001..004).
        let name = self.input_validator.normalized_name(&command);
        let description = self.input_validator.normalized_description(&command)?;
        let UploadMeditationCommand {
            name: raw_name,
            program_id,
            media,
            ..
        } = command;
        let program = self
            .programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        let (stored_media, duration_ms, stored) = self.sniff_and_convert(media).await?;
        let blob = match self
            .store
            .put_staged(
                stored_media.clone(),
                &program.folder,
                &raw_name,
                stored.extension,
            )
            .await
        {
            Ok(blob) => blob,
            Err(error) => {
                // The converted output may be a different file than the upload.
                let _ = self.staging.discard(&stored_media).await;
                return Err(ApplicationError::Storage(error));
            }
        };
        let meditation = match self
            .meditations
            .insert(
                name,
                stored.content_type,
                blob.size_bytes,
                blob.storage_key.clone(),
                duration_ms,
                self.clock.now().await,
                description,
            )
            .await
        {
            Ok(meditation) => meditation,
            Err(error) => {
                // The row is the source of truth: with no row, the blob must not
                // survive either. Best effort — the insert error is primary.
                let _ = self.store.delete(&blob.storage_key).await;
                return Err(ApplicationError::Repository(error));
            }
        };
        self.add_membership_or_roll_back(program_id, &meditation, &blob.storage_key)
            .await?;
        Ok(meditation)
    }

    async fn sniff_and_convert(
        &self,
        media: StagedMedia,
    ) -> Result<(StagedMedia, i64, StoredAudio), ApplicationError> {
        let prefix = self
            .staging
            .read_prefix(&media, SNIFF_PREFIX_BYTES)
            .await
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        let format = self
            .validator
            .sniff(&prefix)
            .await
            .map_err(|_| ApplicationError::UnsupportedMedia)?;
        let input = media.clone();
        let (converted, duration_ms, stored) = self
            .converter
            .convert(media, format)
            .await
            .map_err(map_convert_error)?;
        if converted.address != input.address {
            let _ = self.staging.discard(&input).await;
        }
        Ok((converted, duration_ms, stored))
    }

    async fn add_membership_or_roll_back(
        &self,
        program_id: i64,
        meditation: &Meditation,
        storage_key: &str,
    ) -> Result<(), ApplicationError> {
        let Err(error) = self
            .memberships
            .add(program_id, meditation.id(), self.clock.now().await)
            .await
        else {
            return Ok(());
        };
        // No membership means the meditation is unreachable through any program
        // (upload requires a home program) — undo the row and the blob rather
        // than leave an orphaned, unfiled meditation with a successful-looking
        // response.
        let _ = self.meditations.delete(meditation.id()).await;
        let _ = self.store.delete(storage_key).await;
        Err(ApplicationError::Repository(error))
    }
}

fn map_convert_error(error: ConvertError) -> ApplicationError {
    match error {
        ConvertError::Decode => ApplicationError::UnsupportedMedia,
        ConvertError::Unavailable => {
            ApplicationError::Storage("the Opus encoder is unavailable".to_owned())
        }
    }
}

#[cfg(test)]
#[path = "upload_meditation_command_handler.tests.rs"]
mod tests;
