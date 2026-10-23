use crate::commands::create_category::{
    CreateCategory, CreateCategoryHandler, CreateCategoryService,
};
use crate::commands::delete_category::{
    DeleteCategory, DeleteCategoryHandler, DeleteCategoryService,
};
use crate::commands::set_program_category::{
    SetProgramCategory, SetProgramCategoryHandler, SetProgramCategoryService,
};
use crate::ports::ProgramRepository;
use crate::test_support::TestDeps;

#[tokio::test]
async fn deleting_a_category_leaves_its_programs_uncategorized() {
    let deps = TestDeps::new();
    let category = CreateCategoryService::new(deps.categories.clone(), deps.clock.clone())
        .handle(CreateCategory {
            name: "Evening".into(),
        })
        .await
        .expect("category");
    let program_id = deps.seed_program("Wind-down").await;
    SetProgramCategoryService::new(deps.programs.clone(), deps.categories.clone())
        .handle(SetProgramCategory {
            program_id,
            category_id: Some(category.id),
        })
        .await
        .expect("assign");

    DeleteCategoryService::new(deps.categories.clone(), deps.programs.clone())
        .handle(DeleteCategory {
            category_id: category.id,
        })
        .await
        .expect("delete category");

    let stored = deps
        .programs
        .find_by_id(program_id)
        .await
        .unwrap()
        .expect("program survives");
    assert_eq!(stored.category_id, None);
}
