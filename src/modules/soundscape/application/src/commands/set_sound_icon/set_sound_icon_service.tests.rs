use super::*;
use crate::ports::media::ImageNormalizationError;
use crate::test_support::{sample_sound, FakeImageNormalizer, InMemorySounds, RecordingBlobStore};
use soundscape_domain::{SoundIcon, MAX_ICON_BYTES};

const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn service(
    sounds: Arc<InMemorySounds>,
    store: Arc<RecordingBlobStore>,
    normalized: Result<Vec<u8>, ImageNormalizationError>,
) -> SetSoundIconService {
    SetSoundIconService::new(sounds, store, Arc::new(FakeImageNormalizer(normalized)))
}

#[tokio::test]
async fn stores_the_normalized_icon_on_the_sound() {
    let sounds = Arc::new(InMemorySounds::with(vec![sample_sound(1)]));
    let store = Arc::new(RecordingBlobStore::default());

    let sound = service(sounds, store.clone(), Ok(vec![1, 2, 3]))
        .handle(SetSoundIcon {
            id: 1,
            bytes: PNG_MAGIC.to_vec(),
        })
        .await
        .unwrap();

    let icon = sound.icon.expect("icon is set");
    assert_eq!(icon.content_type, "image/webp");
    assert_eq!(store.put_keys(), vec![icon.storage_key]);
}

#[tokio::test]
async fn replacing_an_icon_deletes_the_previous_blob() {
    let mut with_icon = sample_sound(1);
    with_icon.icon = Some(SoundIcon {
        storage_key: "old-icon".into(),
        content_type: "image/webp".into(),
    });
    let store = Arc::new(RecordingBlobStore::default());

    service(
        Arc::new(InMemorySounds::with(vec![with_icon])),
        store.clone(),
        Ok(vec![1]),
    )
    .handle(SetSoundIcon {
        id: 1,
        bytes: PNG_MAGIC.to_vec(),
    })
    .await
    .unwrap();

    assert_eq!(store.deleted_keys(), vec!["old-icon".to_owned()]);
}

#[tokio::test]
async fn oversized_icon_is_rejected_before_any_write() {
    let store = Arc::new(RecordingBlobStore::default());
    let result = service(
        Arc::new(InMemorySounds::with(vec![sample_sound(1)])),
        store.clone(),
        Ok(vec![1]),
    )
    .handle(SetSoundIcon {
        id: 1,
        bytes: vec![0; MAX_ICON_BYTES + 1],
    })
    .await;
    assert_eq!(result, Err(ApplicationError::PayloadTooLarge));
    assert!(store.put_keys().is_empty());
}

#[tokio::test]
async fn non_image_bytes_are_unsupported_media() {
    let result = service(
        Arc::new(InMemorySounds::with(vec![sample_sound(1)])),
        Arc::new(RecordingBlobStore::default()),
        Ok(vec![1]),
    )
    .handle(SetSoundIcon {
        id: 1,
        bytes: b"<svg></svg>".to_vec(),
    })
    .await;
    assert_eq!(result, Err(ApplicationError::UnsupportedMedia));
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = service(
        Arc::new(InMemorySounds::default()),
        Arc::new(RecordingBlobStore::default()),
        Ok(vec![1]),
    )
    .handle(SetSoundIcon {
        id: 1,
        bytes: PNG_MAGIC.to_vec(),
    })
    .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn normalizer_failure_is_mapped() {
    let result = service(
        Arc::new(InMemorySounds::with(vec![sample_sound(1)])),
        Arc::new(RecordingBlobStore::default()),
        Err(ImageNormalizationError::Busy),
    )
    .handle(SetSoundIcon {
        id: 1,
        bytes: PNG_MAGIC.to_vec(),
    })
    .await;
    assert_eq!(result, Err(ApplicationError::Busy));
}
