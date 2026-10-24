use super::ListProgramMembersQueryValidator;
use crate::queries::list_program_members::ListProgramMembersQuery;

#[test]
fn accepts_the_query() {
    let query = ListProgramMembersQuery { program_id: 1 };

    assert!(ListProgramMembersQueryValidator.validate(&query).is_ok());
}
