//! In-memory fakes for the soundscape application ports, used by the service
//! tests (presentation router tests keep their own fakes).

mod fake_audio_validator;
mod fake_image_normalizer;
mod fake_opus_converter;
mod fake_upload_staging;
mod fixed_clock;
mod in_memory_categories;
mod in_memory_mixes;
mod in_memory_sounds;
mod recording_blob_store;
mod sample_sound;

pub use fake_audio_validator::FakeAudioValidator;
pub use fake_image_normalizer::FakeImageNormalizer;
pub use fake_opus_converter::FakeOpusConverter;
pub use fake_upload_staging::FakeUploadStaging;
pub use fixed_clock::FixedClock;
pub use in_memory_categories::InMemoryCategories;
pub use in_memory_mixes::InMemoryMixes;
pub use in_memory_sounds::InMemorySounds;
pub use recording_blob_store::RecordingBlobStore;
pub use sample_sound::{sample_epoch, sample_sound};
