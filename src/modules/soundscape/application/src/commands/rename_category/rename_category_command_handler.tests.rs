use super::*;
use crate::test_support::InMemoryCategories;

fn command(name: &str) -> RenameCategoryCommand {
    RenameCategoryCommand {
        id: 1,
        name: name.into(),
    }
}

#[tokio::test]
async fn renames_an_existing_category() {
    let categories = InMemoryCategories::with(vec![SoundCategory::new(1, "Old", "old").unwrap()]);
    let renamed = RenameCategoryCommandHandler::new(Arc::new(categories))
        .handle(command("Nature"))
        .await
        .unwrap();
    assert_eq!(renamed.name, "Nature");
}

#[tokio::test]
async fn blank_name_is_rejected() {
    let result = RenameCategoryCommandHandler::new(Arc::new(InMemoryCategories::default()))
        .handle(command("  "))
        .await;
    assert_eq!(result, Err(ApplicationError::InvalidInput("category name")));
}

#[tokio::test]
async fn unknown_category_is_not_found() {
    let result = RenameCategoryCommandHandler::new(Arc::new(InMemoryCategories::default()))
        .handle(command("Nature"))
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn unique_violation_becomes_duplicate() {
    let categories =
        InMemoryCategories::failing("UNIQUE constraint failed: soundscape_categories.name");
    let result = RenameCategoryCommandHandler::new(Arc::new(categories))
        .handle(command("Nature"))
        .await;
    assert_eq!(result, Err(ApplicationError::Duplicate));
}

#[tokio::test]
async fn other_repository_failure_is_reported() {
    let result = RenameCategoryCommandHandler::new(Arc::new(InMemoryCategories::failing("boom")))
        .handle(command("Nature"))
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
