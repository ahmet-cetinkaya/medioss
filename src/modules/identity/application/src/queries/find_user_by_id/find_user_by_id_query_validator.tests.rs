use kernel::UserId;

use super::{FindUserByIdQuery, FindUserByIdQueryValidator};

#[test]
fn accepts_the_query() {
    assert!(FindUserByIdQueryValidator
        .validate(&FindUserByIdQuery { id: UserId::new(1) })
        .is_ok());
}
