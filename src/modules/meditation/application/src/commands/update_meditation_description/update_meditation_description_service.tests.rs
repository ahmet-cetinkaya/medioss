use std::sync::Arc;

use crate::commands::update_meditation_description::{
    UpdateMeditationDescription, UpdateMeditationDescriptionHandler,
    UpdateMeditationDescriptionService,
};
use crate::commands::upload_meditation::{
    UploadMeditation, UploadMeditationHandler, UploadMeditationService,
};
use crate::ports::MeditationRepository;
use crate::test_support::{FakeConverter, FakeValidator, TestDeps};
use crate::ApplicationError;

fn update(meditation_id: i64, description: Option<&str>) -> UpdateMeditationDescription {
    UpdateMeditationDescription {
        meditation_id,
        description: description.map(str::to_owned),
    }
}

#[tokio::test]
async fn description_is_stored_on_upload_and_updatable() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let uploaded = UploadMeditationService::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: false }),
        deps.store.clone(),
        deps.clock.clone(),
    )
    .handle(UploadMeditation {
        name: "Described".into(),
        program_id,
        bytes: b"OggS....OpusHead".to_vec(),
        description: Some(" hello ".into()),
    })
    .await
    .unwrap();
    assert_eq!(uploaded.description(), Some("hello"));

    let service = UpdateMeditationDescriptionService::new(deps.meditations.clone());
    let updated = service
        .handle(update(uploaded.id(), Some("new")))
        .await
        .unwrap();
    assert_eq!(updated.description(), Some("new"));
    let cleared = service
        .handle(update(uploaded.id(), Some("")))
        .await
        .unwrap();
    assert_eq!(cleared.description(), None);
    let stored = deps
        .meditations
        .find_by_id(uploaded.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.description(), None);
}

#[tokio::test]
async fn description_update_rejects_overlong_text_and_unknown_ids() {
    let deps = TestDeps::new();
    let seeded = deps.seeded_meditation().await;
    let service = UpdateMeditationDescriptionService::new(deps.meditations.clone());
    let long = "a".repeat(2001);
    assert_eq!(
        service.handle(update(seeded.id(), Some(&long))).await,
        Err(ApplicationError::InvalidDescription)
    );
    assert_eq!(
        service.handle(update(999, Some("x"))).await,
        Err(ApplicationError::NotFound)
    );
}
