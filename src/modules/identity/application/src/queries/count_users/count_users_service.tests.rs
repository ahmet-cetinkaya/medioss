use std::sync::Arc;

use super::{CountUsers, CountUsersHandler, CountUsersService};
use crate::test_support::{block_on, FakeRepository};

#[test]
fn count_query_returns_the_repository_count() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());

    assert_eq!(
        block_on(CountUsersService::new(repository).handle(CountUsers)).unwrap(),
        1
    );
}
