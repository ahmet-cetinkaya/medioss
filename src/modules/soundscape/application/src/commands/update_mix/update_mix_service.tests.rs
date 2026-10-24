use super::*;
use crate::ports::{MixRepository, NewMix};
use crate::test_support::{sample_epoch, sample_sound, FixedClock, InMemoryMixes, InMemorySounds};
use kernel::UserId;
use soundscape_domain::MixItem;

const OWNER: i64 = 7;

fn item(sound_id: i64, position: i64) -> MixItem {
    MixItem {
        sound_id,
        volume: 50,
        position,
    }
}

async fn service_with_existing_mix() -> (UpdateMixService, Arc<InMemoryMixes>) {
    let mixes = Arc::new(InMemoryMixes::default());
    mixes
        .create(NewMix {
            user_id: UserId::new(OWNER),
            name: "Old".into(),
            items: vec![item(1, 0)],
            now: sample_epoch(),
        })
        .await
        .unwrap();
    let service = UpdateMixService::new(
        mixes.clone(),
        Arc::new(InMemorySounds::with(vec![sample_sound(1), sample_sound(2)])),
        Arc::new(FixedClock(sample_epoch())),
    );
    (service, mixes)
}

fn command(user: i64, name: &str, items: Vec<MixItem>) -> UpdateMix {
    UpdateMix {
        id: 1,
        user_id: UserId::new(user),
        name: name.into(),
        items,
    }
}

#[tokio::test]
async fn replaces_name_and_items_of_own_mix() {
    let (service, mixes) = service_with_existing_mix().await;

    let mix = service
        .handle(command(OWNER, " New ", vec![item(2, 0)]))
        .await
        .unwrap();

    assert_eq!(mix.name, "New");
    assert_eq!(mixes.snapshot()[0].items, vec![item(2, 0)]);
}

#[tokio::test]
async fn another_users_mix_is_not_found_and_untouched() {
    let (service, mixes) = service_with_existing_mix().await;

    let result = service.handle(command(8, "Hijack", vec![item(2, 0)])).await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    assert_eq!(mixes.snapshot()[0].name, "Old");
}

#[tokio::test]
async fn invalid_name_and_items_are_rejected() {
    let (service, _) = service_with_existing_mix().await;
    assert_eq!(
        service.handle(command(OWNER, " ", vec![item(1, 0)])).await,
        Err(ApplicationError::InvalidInput("mix name"))
    );
    assert_eq!(
        service.handle(command(OWNER, "Ok", vec![])).await,
        Err(ApplicationError::InvalidInput("mix items"))
    );
}

#[tokio::test]
async fn unknown_sound_is_not_found() {
    let (service, _) = service_with_existing_mix().await;
    let result = service
        .handle(command(OWNER, "Ok", vec![item(42, 0)]))
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let service = UpdateMixService::new(
        Arc::new(InMemoryMixes::failing("boom")),
        Arc::new(InMemorySounds::with(vec![sample_sound(1)])),
        Arc::new(FixedClock(sample_epoch())),
    );
    let result = service.handle(command(OWNER, "Ok", vec![item(1, 0)])).await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
