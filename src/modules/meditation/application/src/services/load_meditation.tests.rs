use super::load_meditation;
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn loads_the_stored_meditation() {
    let deps = TestDeps::new();
    let seeded = deps.seeded_meditation().await;

    let loaded = load_meditation(deps.meditations.as_ref(), seeded.id())
        .await
        .expect("loaded");

    assert_eq!(loaded.id(), seeded.id());
}

#[tokio::test]
async fn missing_meditation_is_not_found() {
    let deps = TestDeps::new();

    let result = load_meditation(deps.meditations.as_ref(), 9999).await;

    assert_eq!(result.err(), Some(ApplicationError::NotFound));
}
