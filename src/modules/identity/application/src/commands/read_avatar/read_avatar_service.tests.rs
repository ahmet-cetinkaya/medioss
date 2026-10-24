use std::sync::Arc;

use identity_domain::{AvatarRef, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{ReadAvatar, ReadAvatarHandler, ReadAvatarService};
use crate::{
    ports::Clock as _,
    test_support::{block_on, FakeAvatarStore, FakeRepository, FixedClock},
    ApplicationError,
};

fn user(id: i64, role: Role) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse(format!("user{id}@example.com")).unwrap(),
        PasswordHash::from_stored("hash".to_owned()),
        role,
        FixedClock::new().now(),
    )
}

fn store_avatar(repository: &FakeRepository, ext: &str) {
    let changed = repository
        .user_by_id(UserId::new(2))
        .map(|user| user.with_avatar(Some(AvatarRef::from_extension(ext).unwrap())))
        .unwrap();
    repository.seed_user(changed);
}

#[test]
fn read_allows_owner_and_admin_but_not_other_members() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    repository.seed_user(user(1, Role::Admin));
    repository.seed_user(user(3, Role::Member));
    store_avatar(&repository, "webp");
    let store = Arc::new(FakeAvatarStore::accepting());
    let service = ReadAvatarService::new(repository, store);

    let owner = Actor::new(UserId::new(2), Role::Member);
    let admin = Actor::new(UserId::new(1), Role::Admin);
    let other = Actor::new(UserId::new(3), Role::Member);

    assert!(block_on(service.handle(ReadAvatar {
        requested: UserId::new(2),
        reader: owner,
    }))
    .is_ok());
    assert!(block_on(service.handle(ReadAvatar {
        requested: UserId::new(2),
        reader: admin,
    }))
    .is_ok());
    assert_eq!(
        block_on(service.handle(ReadAvatar {
            requested: UserId::new(2),
            reader: other,
        }))
        .unwrap_err(),
        ApplicationError::Forbidden
    );
}

#[test]
fn read_of_a_user_without_an_avatar_is_not_found() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    let store = Arc::new(FakeAvatarStore::accepting());
    let service = ReadAvatarService::new(repository, store);
    let owner = Actor::new(UserId::new(2), Role::Member);

    assert_eq!(
        block_on(service.handle(ReadAvatar {
            requested: UserId::new(2),
            reader: owner,
        }))
        .unwrap_err(),
        ApplicationError::UserNotFound
    );
}
