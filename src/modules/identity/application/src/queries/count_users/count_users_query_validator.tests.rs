use super::{CountUsersQuery, CountUsersQueryValidator};

#[test]
fn accepts_the_query() {
    assert!(CountUsersQueryValidator.validate(&CountUsersQuery).is_ok());
}
