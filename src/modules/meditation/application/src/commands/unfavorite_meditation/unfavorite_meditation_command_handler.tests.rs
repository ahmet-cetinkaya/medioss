use super::{UnfavoriteMeditationCommand, UnfavoriteMeditationCommandHandler};
use crate::commands::favorite_meditation::{
    FavoriteMeditationCommand, FavoriteMeditationCommandHandler,
};
use crate::ports::CommandHandler;
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> UnfavoriteMeditationCommandHandler {
    UnfavoriteMeditationCommandHandler::new(deps.meditations.clone(), deps.favorites.clone())
}

async fn favorite(deps: &TestDeps, user_id: kernel::UserId, meditation_id: i64) {
    FavoriteMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.favorites.clone(),
        deps.clock.clone(),
    )
    .handle(FavoriteMeditationCommand {
        user_id,
        meditation_id,
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn removes_only_the_callers_favorite() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    favorite(&deps, user_a(), meditation.id()).await;
    favorite(&deps, user_b(), meditation.id()).await;

    service(&deps)
        .handle(UnfavoriteMeditationCommand {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();

    let rows = deps.favorites.0.favorites.lock().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, user_b().value());
}

#[tokio::test]
async fn unfavoriting_a_non_favorite_succeeds() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    service(&deps)
        .handle(UnfavoriteMeditationCommand {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn unknown_meditation_is_not_found() {
    let deps = TestDeps::new();

    let error = service(&deps)
        .handle(UnfavoriteMeditationCommand {
            user_id: user_a(),
            meditation_id: 999,
        })
        .await
        .unwrap_err();

    assert_eq!(error, ApplicationError::NotFound);
}
