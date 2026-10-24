use super::*;
use crate::test_support::{sample_sound, InMemorySounds, RecordingBlobStore};
use soundscape_domain::SoundIcon;

const ICON_KEY: &str = "soundscape/icon-1.webp";

fn sound_with_icon() -> Sound {
    let mut sound = sample_sound(1);
    sound.icon = Some(SoundIcon {
        storage_key: ICON_KEY.into(),
        content_type: "image/webp".into(),
    });
    sound
}

fn service(sounds: InMemorySounds, store: Arc<RecordingBlobStore>) -> ClearSoundIconService {
    ClearSoundIconService::new(Arc::new(sounds), store)
}

#[tokio::test]
async fn clearing_removes_icon_and_deletes_its_blob() {
    let store = Arc::new(RecordingBlobStore::default());
    let sounds = InMemorySounds::with(vec![sound_with_icon()]);

    let sound = service(sounds, store.clone())
        .handle(ClearSoundIcon { id: 1 })
        .await
        .unwrap();

    assert_eq!(sound.icon, None);
    assert_eq!(store.deleted_keys(), vec![ICON_KEY.to_owned()]);
}

#[tokio::test]
async fn clearing_a_sound_without_icon_deletes_nothing() {
    let store = Arc::new(RecordingBlobStore::default());
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);

    let sound = service(sounds, store.clone())
        .handle(ClearSoundIcon { id: 1 })
        .await
        .unwrap();

    assert_eq!(sound.icon, None);
    assert!(store.deleted_keys().is_empty());
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let store = Arc::new(RecordingBlobStore::default());
    let result = service(InMemorySounds::default(), store)
        .handle(ClearSoundIcon { id: 9 })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn blob_delete_failure_does_not_fail_the_command() {
    let store = Arc::new(RecordingBlobStore {
        fail_delete: true,
        ..RecordingBlobStore::default()
    });
    let sounds = InMemorySounds::with(vec![sound_with_icon()]);

    let sound = service(sounds, store)
        .handle(ClearSoundIcon { id: 1 })
        .await
        .unwrap();

    assert_eq!(sound.icon, None);
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let store = Arc::new(RecordingBlobStore::default());
    let result = service(InMemorySounds::failing("boom"), store)
        .handle(ClearSoundIcon { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
