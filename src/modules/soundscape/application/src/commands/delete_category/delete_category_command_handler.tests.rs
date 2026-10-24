use super::*;
use crate::test_support::InMemoryCategories;
use soundscape_domain::SoundCategory;

fn nature() -> SoundCategory {
    SoundCategory::new(1, "Nature", "nature").unwrap()
}

#[tokio::test]
async fn empty_category_is_deleted() {
    let categories = Arc::new(InMemoryCategories::with(vec![nature()]));
    DeleteCategoryCommandHandler::new(categories.clone())
        .handle(DeleteCategoryCommand { id: 1 })
        .await
        .unwrap();
    assert!(categories.categories.lock().unwrap().is_empty());
}

#[tokio::test]
async fn unknown_category_is_not_found() {
    let result = DeleteCategoryCommandHandler::new(Arc::new(InMemoryCategories::default()))
        .handle(DeleteCategoryCommand { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn category_with_sounds_is_in_use_and_kept() {
    let categories = Arc::new(InMemoryCategories {
        in_use_ids: vec![1],
        ..InMemoryCategories::with(vec![nature()])
    });
    let result = DeleteCategoryCommandHandler::new(categories.clone())
        .handle(DeleteCategoryCommand { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::CategoryInUse));
    assert_eq!(categories.categories.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = DeleteCategoryCommandHandler::new(Arc::new(InMemoryCategories::failing("boom")))
        .handle(DeleteCategoryCommand { id: 1 })
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
