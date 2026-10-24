use std::collections::HashSet;

use super::UserId;

#[test]
fn value_returns_the_wrapped_id_including_edge_values() {
    assert_eq!(UserId::new(42).value(), 42);
    assert_eq!(UserId::new(0).value(), 0);
    assert_eq!(UserId::new(i64::MAX).value(), i64::MAX);
}

#[test]
fn ids_are_copy_and_equal_only_when_the_value_matches() {
    let id = UserId::new(3);
    let copy = id;

    assert_eq!(id, copy);
    assert_ne!(id, UserId::new(4));
}

#[test]
fn equal_ids_hash_to_one_set_entry() {
    let ids: HashSet<UserId> = [UserId::new(1), UserId::new(1), UserId::new(2)].into();

    assert_eq!(ids.len(), 2);
}
