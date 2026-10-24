use crate::commands::create_category::{CreateCategoryCommand, CreateCategoryCommandHandler};
use crate::commands::delete_category::{DeleteCategoryCommand, DeleteCategoryCommandHandler};
use crate::commands::set_program_category::{
    SetProgramCategoryCommand, SetProgramCategoryCommandHandler,
};
use crate::ports::{CommandHandler, ProgramRepository};
use crate::test_support::TestDeps;

#[tokio::test]
async fn deleting_a_category_leaves_its_programs_uncategorized() {
    let deps = TestDeps::new();
    let category = CreateCategoryCommandHandler::new(deps.categories.clone(), deps.clock.clone())
        .handle(CreateCategoryCommand {
            name: "Evening".into(),
        })
        .await
        .expect("category");
    let program_id = deps.seed_program("Wind-down").await;
    SetProgramCategoryCommandHandler::new(deps.programs.clone(), deps.categories.clone())
        .handle(SetProgramCategoryCommand {
            program_id,
            category_id: Some(category.id),
        })
        .await
        .expect("assign");

    DeleteCategoryCommandHandler::new(deps.categories.clone(), deps.programs.clone())
        .handle(DeleteCategoryCommand {
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
