use kernel::UserId;

use super::super::get_continue_program_query::GetContinueProgramQuery;
use super::GetContinueProgramQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetContinueProgramQueryValidator
        .validate(&GetContinueProgramQuery {
            user_id: UserId::new(1)
        })
        .is_ok());
}
