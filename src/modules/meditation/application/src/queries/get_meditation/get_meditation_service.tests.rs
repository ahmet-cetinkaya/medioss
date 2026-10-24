use crate::queries::get_meditation::{GetMeditation, GetMeditationHandler, GetMeditationService};
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn returns_the_stored_meditation() {
    let deps = TestDeps::new();
    let seeded = deps.seeded_meditation().await;

    let found = GetMeditationService::new(deps.meditations.clone())
        .handle(GetMeditation {
            meditation_id: seeded.id(),
        })
        .await
        .expect("found");

    assert_eq!(found.id(), seeded.id());
    assert_eq!(found.name(), "Seed");
}

#[tokio::test]
async fn unknown_id_is_not_found() {
    let deps = TestDeps::new();

    let result = GetMeditationService::new(deps.meditations.clone())
        .handle(GetMeditation {
            meditation_id: 9999,
        })
        .await;

    assert_eq!(result.err(), Some(ApplicationError::NotFound));
}
