use super::*;
use crate::test_support::{sample_epoch, sample_sound, FixedClock, InMemoryMixes, InMemorySounds};
use kernel::UserId;
use soundscape_domain::MixItem;

fn item(sound_id: i64, position: i64) -> MixItem {
    MixItem {
        sound_id,
        volume: 50,
        position,
    }
}

fn service(mixes: Arc<InMemoryMixes>) -> CreateMixCommandHandler {
    CreateMixCommandHandler::new(
        mixes,
        Arc::new(InMemorySounds::with(vec![sample_sound(1), sample_sound(2)])),
        Arc::new(FixedClock(sample_epoch())),
    )
}

fn command(name: &str, items: Vec<MixItem>) -> CreateMixCommand {
    CreateMixCommand {
        user_id: UserId::new(7),
        name: name.into(),
        items,
    }
}

#[tokio::test]
async fn creates_a_trimmed_mix_stamped_with_the_clock() {
    let mixes = Arc::new(InMemoryMixes::default());

    let mix = service(mixes.clone())
        .handle(command("  Night  ", vec![item(1, 0), item(2, 1)]))
        .await
        .unwrap();

    assert_eq!(mix.name, "Night");
    assert_eq!(mix.user_id, 7);
    assert_eq!(mix.created_at, sample_epoch());
    assert_eq!(mix.items.len(), 2);
    assert_eq!(mixes.snapshot().len(), 1);
}

#[tokio::test]
async fn blank_name_is_rejected_without_persisting() {
    let mixes = Arc::new(InMemoryMixes::default());
    let result = service(mixes.clone())
        .handle(command("   ", vec![item(1, 0)]))
        .await;
    assert_eq!(result, Err(ApplicationError::InvalidInput("mix name")));
    assert!(mixes.snapshot().is_empty());
}

#[tokio::test]
async fn empty_items_are_rejected() {
    let result = service(Arc::new(InMemoryMixes::default()))
        .handle(command("Night", vec![]))
        .await;
    assert_eq!(result, Err(ApplicationError::InvalidInput("mix items")));
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let result = service(Arc::new(InMemoryMixes::default()))
        .handle(command("Night", vec![item(1, 0), item(99, 1)]))
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = service(Arc::new(InMemoryMixes::failing("boom")))
        .handle(command("Night", vec![item(1, 0)]))
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
