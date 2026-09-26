use super::count_users;
use crate::test_support::{block_on, FakeRepository};

#[test]
fn count_query_returns_the_repository_count() {
    let repository = FakeRepository::with_permanent_admin();

    assert_eq!(block_on(count_users(&repository)).unwrap(), 1);
}
