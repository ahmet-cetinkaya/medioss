use crate::ports::CategoryRepository;
use crate::ports::QueryHandler;
use crate::queries::list_categories::{ListCategoriesQuery, ListCategoriesQueryHandler};
use crate::test_support::TestDeps;

#[tokio::test]
async fn empty_when_no_categories() {
    let deps = TestDeps::new();

    let listed = ListCategoriesQueryHandler::new(deps.categories.clone())
        .handle(ListCategoriesQuery)
        .await
        .expect("list");

    assert!(listed.is_empty());
}

#[tokio::test]
async fn lists_every_category() {
    let deps = TestDeps::new();
    deps.categories
        .insert("Sleep".into(), deps.clock.0)
        .await
        .unwrap();
    deps.categories
        .insert("Focus".into(), deps.clock.0)
        .await
        .unwrap();

    let listed = ListCategoriesQueryHandler::new(deps.categories.clone())
        .handle(ListCategoriesQuery)
        .await
        .expect("list");

    let mut names: Vec<_> = listed.into_iter().map(|category| category.name).collect();
    names.sort();
    assert_eq!(names, vec!["Focus", "Sleep"]);
}
