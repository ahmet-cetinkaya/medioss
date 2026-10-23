use crate::commands::create_category::{
    CreateCategory, CreateCategoryHandler, CreateCategoryService,
};
use crate::commands::rename_category::{
    RenameCategory, RenameCategoryHandler, RenameCategoryService,
};
use crate::test_support::TestDeps;
use crate::ApplicationError;

#[tokio::test]
async fn renaming_to_a_name_inserted_after_the_duplicate_check_is_a_duplicate() {
    let deps = TestDeps::new();
    let create = CreateCategoryService::new(deps.categories.clone(), deps.clock.clone());
    let renamed = create
        .handle(CreateCategory {
            name: "Morning".into(),
        })
        .await
        .expect("renamed category");
    create
        .handle(CreateCategory {
            name: "Evening".into(),
        })
        .await
        .expect("existing category");

    let error = RenameCategoryService::new(deps.categories.clone(), deps.clock.clone())
        .handle(RenameCategory {
            category_id: renamed.id,
            name: "Evening".into(),
        })
        .await
        .expect_err("repository duplicate maps to application error");

    assert_eq!(error, ApplicationError::DuplicateCategory);
}
