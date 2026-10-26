use super::ListFavoriteMeditationsQueryValidator;
use crate::queries::list_favorite_meditations::ListFavoriteMeditationsQuery;
use crate::test_support::user_a;

#[test]
fn accepts_the_query() {
    let query = ListFavoriteMeditationsQuery { user_id: user_a() };

    assert!(ListFavoriteMeditationsQueryValidator
        .validate(&query)
        .is_ok());
}
