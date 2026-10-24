use crate::commands::create_category::{CreateCategoryCommand, CreateCategoryCommandHandler};
use crate::ports::CommandHandler;
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn category_names_must_be_unique_and_nonempty() {
    let deps = TestDeps::new();
    let service = CreateCategoryCommandHandler::new(deps.categories.clone(), deps.clock.clone());
    service
        .handle(CreateCategoryCommand {
            name: "Morning".into(),
        })
        .await
        .expect("first");
    let duplicate = service
        .handle(CreateCategoryCommand {
            name: "Morning".into(),
        })
        .await;
    assert_eq!(duplicate.unwrap_err(), ApplicationError::DuplicateCategory);
    let empty = service
        .handle(CreateCategoryCommand { name: "   ".into() })
        .await;
    assert!(matches!(empty, Err(ApplicationError::InvalidInput(_))));
}
