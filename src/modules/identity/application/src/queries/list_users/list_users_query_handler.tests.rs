use std::sync::Arc;

use identity_domain::{Email, RawPassword, Role, User};
use kernel::UserId;

use super::{ListUsersQuery, ListUsersQueryHandler};
use crate::ports::QueryHandler;
use crate::test_support::{block_on, FakeHasher, FakeRepository, FixedClock};

fn seeded_user(id: i64, email: &str, role: Role) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse(email).unwrap(),
        FakeHasher::hash_for(&RawPassword::parse("Passw0rd!").unwrap()),
        role,
        FixedClock::new().now_value(),
    )
}

#[test]
fn list_query_returns_users_ordered_by_email() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(seeded_user(1, "zoe@example.com", Role::Member));
    repository.seed_user(seeded_user(2, "adam@example.com", Role::Admin));

    let users = block_on(ListUsersQueryHandler::new(repository).handle(ListUsersQuery)).unwrap();
    let emails: Vec<&str> = users.iter().map(|u| u.email().as_str()).collect();
    assert_eq!(emails, vec!["adam@example.com", "zoe@example.com"]);
}

#[test]
fn list_query_on_an_empty_repository_returns_no_users() {
    let repository = Arc::new(FakeRepository::new());
    assert!(
        block_on(ListUsersQueryHandler::new(repository).handle(ListUsersQuery))
            .unwrap()
            .is_empty()
    );
}
