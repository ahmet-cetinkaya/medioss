use super::ListCategoriesQueryValidator;
use crate::queries::list_categories::ListCategoriesQuery;

#[test]
fn accepts_the_query() {
    assert!(ListCategoriesQueryValidator
        .validate(&ListCategoriesQuery)
        .is_ok());
}
