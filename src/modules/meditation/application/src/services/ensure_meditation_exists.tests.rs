use super::ensure_meditation_exists;
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn existing_meditation_is_ok() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    let result = ensure_meditation_exists(deps.meditations.as_ref(), meditation.id()).await;

    assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn missing_meditation_is_not_found() {
    let deps = TestDeps::new();

    let result = ensure_meditation_exists(deps.meditations.as_ref(), 9999).await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
