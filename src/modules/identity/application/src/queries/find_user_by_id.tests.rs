use super::find_user_by_id;
use crate::test_support::{block_on, FakeRepository};
use kernel::UserId;

#[test]
fn finds_a_seeded_user_by_id() {
    let repository = FakeRepository::with_permanent_admin();

    let user = block_on(find_user_by_id(&repository, UserId::new(1))).unwrap();

    assert!(user.is_some());
}

#[test]
fn unknown_id_resolves_to_none() {
    let repository = FakeRepository::with_permanent_admin();

    let user = block_on(find_user_by_id(&repository, UserId::new(999))).unwrap();

    assert!(user.is_none());
}
