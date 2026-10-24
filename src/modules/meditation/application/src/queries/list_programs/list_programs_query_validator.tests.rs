use super::ListProgramsQueryValidator;
use crate::queries::list_programs::ListProgramsQuery;

#[test]
fn accepts_the_query() {
    assert!(ListProgramsQueryValidator
        .validate(&ListProgramsQuery)
        .is_ok());
}
