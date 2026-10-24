use crate::ports::QueryHandler;
use crate::queries::list_programs::{ListProgramsQuery, ListProgramsQueryHandler};
use crate::test_support::TestDeps;

#[tokio::test]
async fn empty_when_no_programs() {
    let deps = TestDeps::new();

    let listed = ListProgramsQueryHandler::new(deps.programs.clone())
        .handle(ListProgramsQuery)
        .await
        .expect("list");

    assert!(listed.is_empty());
}

#[tokio::test]
async fn lists_every_program() {
    let deps = TestDeps::new();
    deps.seed_program("Morning").await;
    deps.seed_program("Evening").await;

    let listed = ListProgramsQueryHandler::new(deps.programs.clone())
        .handle(ListProgramsQuery)
        .await
        .expect("list");

    let mut titles: Vec<_> = listed.into_iter().map(|program| program.title).collect();
    titles.sort();
    assert_eq!(titles, vec!["Evening", "Morning"]);
}
