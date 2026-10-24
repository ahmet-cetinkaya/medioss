use super::*;
use crate::ports::media::{AudioFormat, StoredAudio};
use crate::test_support::{
    FakeAudioValidator, FakeImageNormalizer, FakeOpusConverter, InMemoryCategories, InMemorySounds,
    RecordingBlobStore,
};
use soundscape_domain::SoundCategory;

const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const STORED_BYTES: [u8; 3] = [1, 2, 3];

struct Fixture {
    sounds: Arc<InMemorySounds>,
    store: Arc<RecordingBlobStore>,
    validator: Result<AudioFormat, ()>,
    converter: Result<(Vec<u8>, i64, StoredAudio), String>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            sounds: Arc::new(InMemorySounds::default()),
            store: Arc::new(RecordingBlobStore::default()),
            validator: Ok(AudioFormat::Mp3),
            converter: Ok((
                STORED_BYTES.to_vec(),
                1000,
                StoredAudio {
                    extension: "ogg",
                    content_type: "audio/ogg",
                },
            )),
        }
    }

    fn service(&self) -> CreateSoundService {
        let categories =
            InMemoryCategories::with(vec![SoundCategory::new(1, "Nature", "nature").unwrap()]);
        CreateSoundService::new(
            Arc::new(categories),
            self.sounds.clone(),
            Arc::new(FakeAudioValidator(self.validator)),
            Arc::new(FakeOpusConverter(self.converter.clone())),
            self.store.clone(),
            Arc::new(FakeImageNormalizer(Ok(vec![9, 9]))),
        )
    }
}

fn command(name: &str) -> CreateSound {
    CreateSound {
        name: name.into(),
        category_id: 1,
        bytes: vec![0; 16],
        icon_key: None,
        icon: None,
    }
}

#[tokio::test]
async fn creates_a_sound_from_the_converted_audio() {
    let fixture = Fixture::new();

    let sound = fixture.service().handle(command("Rain")).await.unwrap();

    assert_eq!(sound.name, "Rain");
    assert_eq!(sound.content_type, "audio/ogg");
    assert_eq!(sound.size_bytes, STORED_BYTES.len() as i64);
    assert_eq!(sound.duration_ms, 1000);
    assert_eq!(sound.storage_key, "soundscape/Rain-0.ogg");
    assert_eq!(fixture.sounds.snapshot().len(), 1);
}

#[tokio::test]
async fn stored_name_is_trimmed() {
    let fixture = Fixture::new();
    let sound = fixture.service().handle(command("  Rain  ")).await.unwrap();
    assert_eq!(sound.name, "Rain");
}

#[tokio::test]
async fn keeps_a_valid_icon_key() {
    let fixture = Fixture::new();
    let sound = fixture
        .service()
        .handle(CreateSound {
            icon_key: Some("cloud-rain".into()),
            ..command("Rain")
        })
        .await
        .unwrap();
    assert_eq!(sound.icon_key.as_deref(), Some("cloud-rain"));
}

#[tokio::test]
async fn invalid_icon_key_and_name_are_rejected() {
    let fixture = Fixture::new();
    let bad_key = fixture
        .service()
        .handle(CreateSound {
            icon_key: Some("Bad Key!".into()),
            ..command("Rain")
        })
        .await;
    assert_eq!(bad_key, Err(ApplicationError::InvalidInput("icon key")));
    assert_eq!(
        fixture.service().handle(command("  ")).await,
        Err(ApplicationError::InvalidInput("sound name"))
    );
}

#[tokio::test]
async fn oversized_upload_is_rejected_before_any_write() {
    let fixture = Fixture::new();
    let result = fixture
        .service()
        .handle(CreateSound {
            bytes: vec![0; MAX_UPLOAD_BYTES + 1],
            ..command("Rain")
        })
        .await;
    assert_eq!(result, Err(ApplicationError::PayloadTooLarge));
    assert!(fixture.store.put_keys().is_empty());
}

#[tokio::test]
async fn unknown_category_is_not_found() {
    let fixture = Fixture::new();
    let result = fixture
        .service()
        .handle(CreateSound {
            category_id: 99,
            ..command("Rain")
        })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn unrecognised_audio_is_unsupported_media() {
    let mut fixture = Fixture::new();
    fixture.validator = Err(());
    let result = fixture.service().handle(command("Rain")).await;
    assert_eq!(result, Err(ApplicationError::UnsupportedMedia));
    assert!(fixture.store.put_keys().is_empty());
}

#[tokio::test]
async fn converter_failure_is_a_storage_error() {
    let mut fixture = Fixture::new();
    fixture.converter = Err("ffmpeg failed".into());
    let result = fixture.service().handle(command("Rain")).await;
    assert_eq!(
        result,
        Err(ApplicationError::Storage("ffmpeg failed".into()))
    );
}

#[tokio::test]
async fn zero_duration_audio_is_unsupported_media() {
    let mut fixture = Fixture::new();
    fixture.converter = Ok((
        STORED_BYTES.to_vec(),
        0,
        StoredAudio {
            extension: "ogg",
            content_type: "audio/ogg",
        },
    ));
    let result = fixture.service().handle(command("Rain")).await;
    assert_eq!(result, Err(ApplicationError::UnsupportedMedia));
}

#[tokio::test]
async fn blob_store_failure_is_a_storage_error() {
    let mut fixture = Fixture::new();
    fixture.store = Arc::new(RecordingBlobStore {
        fail_put: true,
        ..RecordingBlobStore::default()
    });
    let result = fixture.service().handle(command("Rain")).await;
    assert_eq!(result, Err(ApplicationError::Storage("put failed".into())));
}

#[tokio::test]
async fn failed_row_insert_deletes_the_stored_audio() {
    let mut fixture = Fixture::new();
    fixture.sounds = Arc::new(InMemorySounds {
        fail_create: true,
        ..InMemorySounds::default()
    });
    let result = fixture.service().handle(command("Rain")).await;
    assert_eq!(
        result,
        Err(ApplicationError::Repository("create failed".into()))
    );
    assert_eq!(fixture.store.deleted_keys(), fixture.store.put_keys());
}

#[tokio::test]
async fn attaches_an_uploaded_icon() {
    let fixture = Fixture::new();
    let sound = fixture
        .service()
        .handle(CreateSound {
            icon: Some(PNG_MAGIC.to_vec()),
            ..command("Rain")
        })
        .await
        .unwrap();
    assert!(sound.icon.is_some());
    assert_eq!(fixture.store.put_keys().len(), 2);
}

#[tokio::test]
async fn unsupported_icon_is_rejected_before_audio_is_stored() {
    let fixture = Fixture::new();
    let result = fixture
        .service()
        .handle(CreateSound {
            icon: Some(b"not an image".to_vec()),
            ..command("Rain")
        })
        .await;
    assert_eq!(result, Err(ApplicationError::UnsupportedMedia));
    assert!(fixture.store.put_keys().is_empty());
}

#[tokio::test]
async fn failed_icon_attach_rolls_back_the_sound() {
    let mut fixture = Fixture::new();
    fixture.sounds = Arc::new(InMemorySounds {
        fail_set_icon: true,
        ..InMemorySounds::default()
    });
    let result = fixture
        .service()
        .handle(CreateSound {
            icon: Some(PNG_MAGIC.to_vec()),
            ..command("Rain")
        })
        .await;
    assert_eq!(
        result,
        Err(ApplicationError::Repository("set_icon failed".into()))
    );
    assert!(fixture.sounds.snapshot().is_empty());
    assert!(fixture
        .store
        .deleted_keys()
        .contains(&"soundscape/Rain-0.ogg".to_owned()));
}
