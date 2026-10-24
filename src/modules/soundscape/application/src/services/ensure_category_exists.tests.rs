use super::*;
use crate::test_support::InMemoryCategories;
use soundscape_domain::SoundCategory;

#[tokio::test]
async fn existing_category_passes() {
    let categories =
        InMemoryCategories::with(vec![SoundCategory::new(1, "Nature", "nature").unwrap()]);
    assert_eq!(ensure_category_exists(&categories, 1).await, Ok(()));
}

#[tokio::test]
async fn missing_category_is_not_found() {
    let result = ensure_category_exists(&InMemoryCategories::default(), 1).await;
    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn repository_failure_is_reported() {
    let result = ensure_category_exists(&InMemoryCategories::failing("boom"), 1).await;
    assert_eq!(result, Err(ApplicationError::Repository("boom".into())));
}
