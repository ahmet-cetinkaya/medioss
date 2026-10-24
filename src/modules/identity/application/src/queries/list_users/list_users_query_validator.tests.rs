use super::{ListUsersQuery, ListUsersQueryValidator};

#[test]
fn accepts_the_query() {
    assert!(ListUsersQueryValidator.validate(&ListUsersQuery).is_ok());
}
