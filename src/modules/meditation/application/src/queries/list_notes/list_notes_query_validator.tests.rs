use super::ListNotesQueryValidator;
use crate::queries::list_notes::ListNotesQuery;
use crate::test_support::user_a;

#[test]
fn accepts_the_query() {
    let query = ListNotesQuery {
        user_id: user_a(),
        meditation_id: 1,
    };

    assert!(ListNotesQueryValidator.validate(&query).is_ok());
}
