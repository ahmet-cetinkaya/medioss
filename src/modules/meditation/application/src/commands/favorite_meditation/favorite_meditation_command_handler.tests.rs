use super::{FavoriteMeditationCommand, FavoriteMeditationCommandHandler};
use crate::ports::CommandHandler;
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> FavoriteMeditationCommandHandler {
    FavoriteMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.favorites.clone(),
        deps.clock.clone(),
    )
}

#[tokio::test]
async fn favorites_an_existing_meditation_for_the_caller_only() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    service(&deps)
        .handle(FavoriteMeditationCommand {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();

    let rows = deps.favorites.0.favorites.lock().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].0, rows[0].1), (user_a().value(), meditation.id()));
    assert_ne!(rows[0].0, user_b().value());
}

#[tokio::test]
async fn favoriting_twice_keeps_one_row() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let command = || FavoriteMeditationCommand {
        user_id: user_a(),
        meditation_id: meditation.id(),
    };

    service(&deps).handle(command()).await.unwrap();
    service(&deps).handle(command()).await.unwrap();

    assert_eq!(deps.favorites.0.favorites.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn unknown_meditation_is_not_found_and_stores_nothing() {
    let deps = TestDeps::new();

    let error = service(&deps)
        .handle(FavoriteMeditationCommand {
            user_id: user_a(),
            meditation_id: 999,
        })
        .await
        .unwrap_err();

    assert_eq!(error, ApplicationError::NotFound);
    assert!(deps.favorites.0.favorites.lock().unwrap().is_empty());
}
