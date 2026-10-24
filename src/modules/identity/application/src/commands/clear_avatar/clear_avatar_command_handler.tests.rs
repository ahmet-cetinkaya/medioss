use std::sync::Arc;

use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{ClearAvatarCommand, ClearAvatarCommandHandler};
use crate::ports::{Clock as _, CommandHandler as _};
use crate::test_support::{block_on, FakeAvatarStore, FakeRepository, FixedClock};

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
fn clear_is_idempotent_and_removes_only_existing_files() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "webp");
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);
    let service = ClearAvatarCommandHandler::new(repository.clone(), store.clone());

    block_on(service.handle(ClearAvatarCommand {
        actor: actor.clone(),
    }))
    .unwrap();
    block_on(service.handle(ClearAvatarCommand { actor })).unwrap();

    assert_eq!(store.removed(), vec![(2, "webp")]);
    let cleared = repository.user_by_id(UserId::new(2)).unwrap();
    assert_eq!(cleared.avatar(), None);
}

#[test]
fn clear_keeps_the_display_name() {
    let repository = Arc::new(FakeRepository::new());
    let named =
        user(2, Role::Member).with_display_name(Some(DisplayName::parse("Ahmet QA").unwrap()));
    repository.seed_user(named);
    store_avatar(&repository, "webp");
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    block_on(
        ClearAvatarCommandHandler::new(repository.clone(), store.clone())
            .handle(ClearAvatarCommand { actor }),
    )
    .unwrap();

    let cleared = repository.user_by_id(UserId::new(2)).unwrap();
    assert_eq!(cleared.display_name().map(|n| n.as_str()), Some("Ahmet QA"));
}

#[test]
fn clear_without_an_avatar_touches_nothing() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    block_on(
        ClearAvatarCommandHandler::new(repository.clone(), store.clone())
            .handle(ClearAvatarCommand { actor }),
    )
    .unwrap();

    assert!(store.removed().is_empty());
}
