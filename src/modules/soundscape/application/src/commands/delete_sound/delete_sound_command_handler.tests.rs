use super::*;
use crate::test_support::{sample_sound, InMemorySounds, RecordingBlobStore};

#[tokio::test]
async fn deletes_the_sound_row_and_its_blob() {
    let sounds = Arc::new(InMemorySounds::with(vec![sample_sound(1)]));
    let store = Arc::new(RecordingBlobStore::default());

    DeleteSoundCommandHandler::new(sounds.clone(), store.clone())
        .handle(DeleteSoundCommand { id: 1 })
        .await
        .unwrap();

    assert!(sounds.snapshot().is_empty());
    assert_eq!(
        store.deleted_keys(),
        vec!["soundscape/sound-1.ogg".to_owned()]
    );
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = DeleteSoundCommandHandler::new(
        Arc::new(InMemorySounds::default()),
        Arc::new(RecordingBlobStore::default()),
    )
    .handle(DeleteSoundCommand { id: 1 })
    .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}
