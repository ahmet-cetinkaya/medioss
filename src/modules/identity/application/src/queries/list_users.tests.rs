use identity_domain::{Email, RawPassword, Role, User};
use kernel::UserId;

use super::list_users;
use crate::test_support::{block_on, FakeHasher, FakeRepository, FixedClock};

#[test]
fn list_query_returns_users_ordered_by_email() {
    let repository = FakeRepository::new();
    repository.seed_user(User::new_permanent(
        UserId::new(1),
        Email::parse("zoe@example.com").unwrap(),
        FakeHasher::hash_for(&RawPassword::parse("Passw0rd!").unwrap()),
        Role::Member,
        FixedClock::new().now_value(),
    ));
    repository.seed_user(User::new_permanent(
        UserId::new(2),
        Email::parse("adam@example.com").unwrap(),
        FakeHasher::hash_for(&RawPassword::parse("Passw0rd!").unwrap()),
        Role::Admin,
        FixedClock::new().now_value(),
    ));

    let users = block_on(list_users(&repository)).unwrap();
    let emails: Vec<&str> = users.iter().map(|u| u.email().as_str()).collect();
    assert_eq!(emails, vec!["adam@example.com", "zoe@example.com"]);
}

#[test]
fn list_query_on_an_empty_repository_returns_no_users() {
    let repository = FakeRepository::new();
    assert!(block_on(list_users(&repository)).unwrap().is_empty());
}
