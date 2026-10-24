use super::ListRecentProgramsQueryValidator;
use crate::queries::list_recent_programs::ListRecentProgramsQuery;
use crate::test_support::user_a;

#[test]
fn accepts_the_query() {
    let query = ListRecentProgramsQuery {
        user_id: user_a(),
        limit: Some(3),
    };

    assert!(ListRecentProgramsQueryValidator.validate(&query).is_ok());
}
