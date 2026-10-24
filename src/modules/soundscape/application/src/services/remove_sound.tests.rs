use super::*;
use crate::test_support::{sample_sound, InMemorySounds, RecordingBlobStore};
use soundscape_domain::SoundIcon;

#[tokio::test]
async fn removes_the_row_and_its_audio_blob() {
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);
    let store = RecordingBlobStore::default();

    remove_sound(&sounds, &store, 1).await.unwrap();

    assert!(sounds.snapshot().is_empty());
    assert_eq!(
        store.deleted_keys(),
        vec!["soundscape/sound-1.ogg".to_owned()]
    );
}

#[tokio::test]
async fn also_removes_the_icon_blob() {
    let mut sound = sample_sound(1);
    sound.icon = Some(SoundIcon {
        storage_key: "icon-key".into(),
        content_type: "image/webp".into(),
    });
    let sounds = InMemorySounds::with(vec![sound]);
    let store = RecordingBlobStore::default();

    remove_sound(&sounds, &store, 1).await.unwrap();

    assert_eq!(
        store.deleted_keys(),
        vec!["icon-key".to_owned(), "soundscape/sound-1.ogg".to_owned()]
    );
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = remove_sound(
        &InMemorySounds::default(),
        &RecordingBlobStore::default(),
        1,
    )
    .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn audio_blob_failure_is_a_storage_error() {
    let store = RecordingBlobStore {
        fail_delete: true,
        ..RecordingBlobStore::default()
    };
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);
    let result = remove_sound(&sounds, &store, 1).await;
    assert_eq!(
        result,
        Err(ApplicationError::Storage("delete failed".into()))
    );
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = remove_sound(
        &InMemorySounds::failing("boom"),
        &RecordingBlobStore::default(),
        1,
    )
    .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
