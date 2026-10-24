use kernel::UserId;

use super::super::get_program_cover_query::GetProgramCoverQuery;
use super::GetProgramCoverQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetProgramCoverQueryValidator
        .validate(&GetProgramCoverQuery {
            requester_id: UserId::new(1),
            program_id: 1
        })
        .is_ok());
}
