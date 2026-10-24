use super::*;
use crate::test_support::{sample_sound, InMemorySounds, RecordingBlobStore};

#[tokio::test]
async fn stores_a_webp_icon_on_the_sound() {
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);
    let store = RecordingBlobStore::default();

    let sound = store_sound_icon(&sounds, &store, 1, vec![1, 2])
        .await
        .unwrap();

    let icon = sound.icon.expect("icon is set");
    assert_eq!(icon.content_type, "image/webp");
    assert_eq!(icon.storage_key, "soundscape/icon-1-0.webp");
    assert!(store.deleted_keys().is_empty());
}

#[tokio::test]
async fn replaced_icon_blob_is_deleted() {
    let mut sound = sample_sound(1);
    sound.icon = Some(SoundIcon {
        storage_key: "old-icon".into(),
        content_type: "image/webp".into(),
    });
    let sounds = InMemorySounds::with(vec![sound]);
    let store = RecordingBlobStore::default();

    store_sound_icon(&sounds, &store, 1, vec![1]).await.unwrap();

    assert_eq!(store.deleted_keys(), vec!["old-icon".to_owned()]);
}

#[tokio::test]
async fn unknown_sound_is_not_found_and_nothing_is_stored() {
    let store = RecordingBlobStore::default();
    let result = store_sound_icon(&InMemorySounds::default(), &store, 1, vec![1]).await;
    assert_eq!(result, Err(ApplicationError::NotFound));
    assert!(store.put_keys().is_empty());
}

#[tokio::test]
async fn blob_failure_is_a_storage_error() {
    let store = RecordingBlobStore {
        fail_put: true,
        ..RecordingBlobStore::default()
    };
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);
    let result = store_sound_icon(&sounds, &store, 1, vec![1]).await;
    assert_eq!(result, Err(ApplicationError::Storage("put failed".into())));
}

#[tokio::test]
async fn failed_icon_update_keeps_the_new_blob() {
    let sounds = InMemorySounds {
        fail_set_icon: true,
        ..InMemorySounds::with(vec![sample_sound(1)])
    };
    let store = RecordingBlobStore::default();

    let result = store_sound_icon(&sounds, &store, 1, vec![1]).await;

    assert_eq!(
        result,
        Err(ApplicationError::Repository("set_icon failed".into()))
    );
    assert!(store.deleted_keys().is_empty());
}
