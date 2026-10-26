use super::ListFavoriteIdsQueryValidator;
use crate::queries::list_favorite_ids::ListFavoriteIdsQuery;
use crate::test_support::user_a;

#[test]
fn accepts_the_query() {
    let query = ListFavoriteIdsQuery { user_id: user_a() };

    assert!(ListFavoriteIdsQueryValidator.validate(&query).is_ok());
}
