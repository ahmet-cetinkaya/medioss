use std::sync::Arc;

use crate::commands::rename_meditation::{RenameMeditationCommand, RenameMeditationCommandHandler};
use crate::commands::upload_meditation::{UploadMeditationCommand, UploadMeditationCommandHandler};
use crate::ports::{CommandHandler, MeditationRepository};
use crate::test_support::{FakeConverter, FakeValidator, TestDeps};
use crate::ApplicationError;

async fn seed_meditation(deps: &TestDeps) -> i64 {
    let program_id = deps.seed_program("Holder Program").await;
    UploadMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: false }),
        deps.store.clone(),
        deps.staging.clone(),
        deps.clock.clone(),
        crate::test_support::TEST_MAX_UPLOAD_BYTES,
    )
    .handle(UploadMeditationCommand {
        name: "Original".into(),
        program_id,
        media: deps.staging.stage(b"OggS....OpusHead"),
        description: Some("kept".into()),
    })
    .await
    .unwrap()
    .id()
}

fn rename(meditation_id: i64, name: &str) -> RenameMeditationCommand {
    RenameMeditationCommand {
        meditation_id,
        name: name.to_owned(),
    }
}

#[tokio::test]
async fn renaming_stores_the_trimmed_name_and_keeps_the_description() {
    let deps = TestDeps::new();
    let id = seed_meditation(&deps).await;
    let handler = RenameMeditationCommandHandler::new(deps.meditations.clone());

    let renamed = handler.handle(rename(id, "  Evening calm ")).await.unwrap();

    assert_eq!(renamed.name(), "Evening calm");
    let stored = deps.meditations.find_by_id(id).await.unwrap().unwrap();
    assert_eq!(stored.name(), "Evening calm");
    assert_eq!(stored.description(), Some("kept"));
}

#[tokio::test]
async fn an_invalid_name_changes_nothing() {
    let deps = TestDeps::new();
    let id = seed_meditation(&deps).await;
    let handler = RenameMeditationCommandHandler::new(deps.meditations.clone());

    let result = handler.handle(rename(id, "   ")).await;

    assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    let stored = deps.meditations.find_by_id(id).await.unwrap().unwrap();
    assert_eq!(stored.name(), "Original");
}

#[tokio::test]
async fn an_unknown_meditation_is_not_found() {
    let deps = TestDeps::new();
    let handler = RenameMeditationCommandHandler::new(deps.meditations.clone());

    let result = handler.handle(rename(999, "Anything")).await;

    assert!(matches!(result, Err(ApplicationError::NotFound)));
}
