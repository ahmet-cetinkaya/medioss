use crate::commands::create_category::{
    CreateCategory, CreateCategoryHandler, CreateCategoryService,
};
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn category_names_must_be_unique_and_nonempty() {
    let deps = TestDeps::new();
    let service = CreateCategoryService::new(deps.categories.clone(), deps.clock.clone());
    service
        .handle(CreateCategory {
            name: "Morning".into(),
        })
        .await
        .expect("first");
    let duplicate = service
        .handle(CreateCategory {
            name: "Morning".into(),
        })
        .await;
    assert_eq!(duplicate.unwrap_err(), ApplicationError::DuplicateCategory);
    let empty = service.handle(CreateCategory { name: "   ".into() }).await;
    assert!(matches!(empty, Err(ApplicationError::InvalidInput(_))));
}
