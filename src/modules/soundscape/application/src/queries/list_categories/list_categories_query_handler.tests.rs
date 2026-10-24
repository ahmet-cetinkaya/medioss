use super::*;
use crate::test_support::InMemoryCategories;

#[tokio::test]
async fn lists_every_category() {
    let categories = InMemoryCategories::with(vec![
        SoundCategory::new(1, "Nature", "nature").unwrap(),
        SoundCategory::new(2, "Urban", "urban").unwrap(),
    ]);
    let listed = ListCategoriesQueryHandler::new(Arc::new(categories))
        .handle(ListCategoriesQuery)
        .await
        .unwrap();
    let names: Vec<_> = listed
        .iter()
        .map(|category| category.name.as_str())
        .collect();
    assert_eq!(names, vec!["Nature", "Urban"]);
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = ListCategoriesQueryHandler::new(Arc::new(InMemoryCategories::failing("boom")))
        .handle(ListCategoriesQuery)
        .await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
