use super::*;
use crate::test_support::{sample_sound, InMemoryCategories, InMemorySounds};
use soundscape_domain::SoundCategory;

fn service(sounds: InMemorySounds) -> UpdateSoundService {
    let categories = InMemoryCategories::with(vec![
        SoundCategory::new(1, "Nature", "nature").unwrap(),
        SoundCategory::new(2, "Urban", "urban").unwrap(),
    ]);
    UpdateSoundService::new(Arc::new(categories), Arc::new(sounds))
}

fn command(name: &str, category_id: i64, icon_key: Option<&str>) -> UpdateSound {
    UpdateSound {
        id: 1,
        name: name.into(),
        category_id,
        icon_key: icon_key.map(str::to_owned),
    }
}

#[tokio::test]
async fn updates_name_category_and_icon_key() {
    let sound = service(InMemorySounds::with(vec![sample_sound(1)]))
        .handle(command("Storm", 2, Some("cloud-rain")))
        .await
        .unwrap();
    assert_eq!(sound.name, "Storm");
    assert_eq!(sound.category_id, 2);
    assert_eq!(sound.icon_key.as_deref(), Some("cloud-rain"));
}

#[tokio::test]
async fn blank_icon_key_clears_it() {
    let mut existing = sample_sound(1);
    existing.icon_key = Some("rain".into());
    let sound = service(InMemorySounds::with(vec![existing]))
        .handle(command("Rain", 1, Some("  ")))
        .await
        .unwrap();
    assert_eq!(sound.icon_key, None);
}

#[tokio::test]
async fn invalid_name_is_rejected() {
    let result = service(InMemorySounds::with(vec![sample_sound(1)]))
        .handle(command(" ", 1, None))
        .await;
    assert_eq!(result, Err(ApplicationError::InvalidInput("sound name")));
}

#[tokio::test]
async fn invalid_icon_key_is_rejected() {
    let result = service(InMemorySounds::with(vec![sample_sound(1)]))
        .handle(command("Rain", 1, Some("Bad Key!")))
        .await;
    assert_eq!(result, Err(ApplicationError::InvalidInput("icon key")));
}

#[tokio::test]
async fn unknown_category_is_not_found() {
    let result = service(InMemorySounds::with(vec![sample_sound(1)]))
        .handle(command("Rain", 99, None))
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = service(InMemorySounds::default())
        .handle(command("Rain", 1, None))
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = service(InMemorySounds::failing("boom"))
        .handle(command("Rain", 1, None))
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
