use crate::commands::create_category::{CreateCategoryCommand, CreateCategoryCommandHandler};
use crate::commands::rename_category::{RenameCategoryCommand, RenameCategoryCommandHandler};
use crate::ports::CommandHandler;
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn renaming_to_a_name_inserted_after_the_duplicate_check_is_a_duplicate() {
    let deps = TestDeps::new();
    let create = CreateCategoryCommandHandler::new(deps.categories.clone(), deps.clock.clone());
    let renamed = create
        .handle(CreateCategoryCommand {
            name: "Morning".into(),
        })
        .await
        .expect("renamed category");
    create
        .handle(CreateCategoryCommand {
            name: "Evening".into(),
        })
        .await
        .expect("existing category");

    let error = RenameCategoryCommandHandler::new(deps.categories.clone(), deps.clock.clone())
        .handle(RenameCategoryCommand {
            category_id: renamed.id,
            name: "Evening".into(),
        })
        .await
        .expect_err("repository duplicate maps to application error");

    assert_eq!(error, ApplicationError::DuplicateCategory);
}
