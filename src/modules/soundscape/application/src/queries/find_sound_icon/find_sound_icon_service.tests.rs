use super::*;
use crate::test_support::{sample_sound, InMemorySounds};

fn service(sounds: InMemorySounds) -> FindSoundIconService {
    FindSoundIconService::new(Arc::new(sounds))
}

#[tokio::test]
async fn returns_the_uploaded_icon() {
    let icon = SoundIcon {
        storage_key: "soundscape/icon-1.webp".into(),
        content_type: "image/webp".into(),
    };
    let mut sound = sample_sound(1);
    sound.icon = Some(icon.clone());

    let found = service(InMemorySounds::with(vec![sound]))
        .handle(FindSoundIcon { id: 1 })
        .await
        .unwrap();

    assert_eq!(found, icon);
}

#[tokio::test]
async fn sound_without_icon_is_not_found() {
    let result = service(InMemorySounds::with(vec![sample_sound(1)]))
        .handle(FindSoundIcon { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = service(InMemorySounds::default())
        .handle(FindSoundIcon { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = service(InMemorySounds::failing("boom"))
        .handle(FindSoundIcon { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
