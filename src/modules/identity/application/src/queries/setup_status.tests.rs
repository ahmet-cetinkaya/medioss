use super::is_setup_required;
use crate::test_support::{block_on, FakeRepository};

#[test]
fn setup_is_required_only_for_an_empty_repository() {
    let empty = FakeRepository::new();
    let initialized = FakeRepository::with_permanent_admin();

    assert!(block_on(is_setup_required(&empty)).unwrap());
    assert!(!block_on(is_setup_required(&initialized)).unwrap());
}
