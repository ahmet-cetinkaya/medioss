use std::sync::Arc;

use kernel::UserId;

use super::{FindUserByIdQuery, FindUserByIdQueryHandler};
use crate::ports::QueryHandler;
use crate::test_support::{block_on, FakeRepository};

#[test]
fn finds_a_seeded_user_by_id() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());

    let user = block_on(
        FindUserByIdQueryHandler::new(repository).handle(FindUserByIdQuery { id: UserId::new(1) }),
    )
    .unwrap();

    assert!(user.is_some());
}

#[test]
fn unknown_id_resolves_to_none() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());

    let user = block_on(
        FindUserByIdQueryHandler::new(repository).handle(FindUserByIdQuery {
            id: UserId::new(999),
        }),
    )
    .unwrap();

    assert!(user.is_none());
}
