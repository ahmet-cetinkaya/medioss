use super::*;

#[test]
fn accepts_the_query() {
    assert_eq!(
        ListCategoriesQueryValidator.validate(&ListCategoriesQuery),
        Ok(())
    );
}
