use std::sync::Arc;

use super::{CountUsersQuery, CountUsersQueryHandler};
use crate::ports::QueryHandler;
use crate::test_support::{block_on, FakeRepository};

#[test]
fn count_query_returns_the_repository_count() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());

    assert_eq!(
        block_on(CountUsersQueryHandler::new(repository).handle(CountUsersQuery)).unwrap(),
        1
    );
}
