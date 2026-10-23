use std::sync::Arc;

use super::upload_meditation::UploadMeditation;
use super::upload_meditation::MAX_UPLOAD_BYTES;
use super::upload_meditation_handler::UploadMeditationHandler;
use crate::ports::{
    AudioValidator, BlobStore, Clock, MeditationRepository, MembershipRepository, OpusConverter,
    ProgramRepository,
};
use crate::ports::{ConvertError, StoredAudio};
use crate::services::{validate_description, validate_name};
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct UploadMeditationService {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
    validator: Arc<dyn AudioValidator>,
    converter: Arc<dyn OpusConverter>,
    store: Arc<dyn BlobStore>,
    clock: Arc<dyn Clock>,
}

impl UploadMeditationService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
        validator: Arc<dyn AudioValidator>,
        converter: Arc<dyn OpusConverter>,
        store: Arc<dyn BlobStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            programs,
            validator,
            converter,
            store,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl UploadMeditationHandler for UploadMeditationService {
    async fn handle(&self, command: UploadMeditation) -> Result<Meditation, ApplicationError> {
        // Garbage bytes, oversized payloads, and conversion failures leave
        // nothing stored (spec FR-001..004).
        let UploadMeditation {
            name,
            program_id,
            bytes,
            description,
        } = command;
        validate_name(&name)?;
        let description = validate_description(description.as_deref())?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(ApplicationError::PayloadTooLarge);
        }
        let program = self
            .programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        let (stored_bytes, duration_ms, stored) = self.sniff_and_convert(bytes).await?;
        let blob = self
            .store
            .put(&program.folder, &name, stored.extension, &stored_bytes)
            .await
            .map_err(ApplicationError::Storage)?;
        let meditation = match self
            .meditations
            .insert(
                name.trim().to_owned(),
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
}

impl UploadMeditationService {
    async fn sniff_and_convert(
        &self,
        bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, i64, StoredAudio), ApplicationError> {
        let format = self
            .validator
            .sniff(&bytes)
            .await
            .map_err(|_| ApplicationError::UnsupportedMedia)?;
        self.converter
            .convert(bytes, format)
            .await
            .map_err(map_convert_error)
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
#[path = "upload_meditation_service.tests.rs"]
mod tests;
