use super::*;
use crate::test_support::{sample_sound, InMemorySounds};

fn item(sound_id: i64) -> MixItem {
    MixItem {
        sound_id,
        volume: 50,
        position: 0,
    }
}

#[tokio::test]
async fn all_existing_sounds_pass() {
    let sounds = InMemorySounds::with(vec![sample_sound(1), sample_sound(2)]);
    assert_eq!(
        ensure_sounds_exist(&sounds, &[item(1), item(2)]).await,
        Ok(())
    );
}

#[tokio::test]
async fn no_items_pass_trivially() {
    assert_eq!(
        ensure_sounds_exist(&InMemorySounds::default(), &[]).await,
        Ok(())
    );
}

#[tokio::test]
async fn one_missing_sound_is_not_found() {
    let sounds = InMemorySounds::with(vec![sample_sound(1)]);
    let result = ensure_sounds_exist(&sounds, &[item(1), item(2)]).await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = ensure_sounds_exist(&InMemorySounds::failing("boom"), &[item(1)]).await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
