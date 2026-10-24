use std::sync::Arc;

use identity_domain::{DisplayName, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{SetDisplayNameCommand, SetDisplayNameCommandHandler};
use crate::ports::CommandHandler as _;
use crate::{
    ports::Clock as _,
    test_support::{block_on, FakeRepository, FixedClock},
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

fn stored_name(repository: &FakeRepository) -> Option<String> {
    repository
        .user_by_id(UserId::new(2))
        .and_then(|user| user.display_name().map(|name| name.as_str().to_owned()))
}

#[test]
fn sets_trims_and_persists_the_display_name() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));

    let updated = block_on(
        SetDisplayNameCommandHandler::new(repository.clone()).handle(SetDisplayNameCommand {
            user_id: UserId::new(2),
            display_name: Some(DisplayName::parse("  Ada  ").unwrap()),
        }),
    )
    .unwrap();

    assert_eq!(
        updated.display_name().map(|name| name.as_str()),
        Some("Ada")
    );
    assert_eq!(stored_name(&repository).as_deref(), Some("Ada"));
}

#[test]
fn empty_name_clears_an_existing_one() {
    let repository = Arc::new(FakeRepository::new());
    let named = user(2, Role::Member).with_display_name(Some(DisplayName::parse("Ada").unwrap()));
    repository.seed_user(named);

    block_on(
        SetDisplayNameCommandHandler::new(repository.clone()).handle(SetDisplayNameCommand {
            user_id: UserId::new(2),
            display_name: None,
        }),
    )
    .unwrap();

    assert_eq!(stored_name(&repository), None);
}

#[test]
fn keeps_the_avatar_untouched() {
    let repository = Arc::new(FakeRepository::new());
    let with_avatar = user(2, Role::Member).with_avatar(Some(identity_domain::AvatarRef::Webp));
    repository.seed_user(with_avatar);

    block_on(
        SetDisplayNameCommandHandler::new(repository.clone()).handle(SetDisplayNameCommand {
            user_id: UserId::new(2),
            display_name: Some(DisplayName::parse("Ada").unwrap()),
        }),
    )
    .unwrap();

    assert_eq!(
        repository
            .user_by_id(UserId::new(2))
            .map(|user| user.avatar()),
        Some(Some(identity_domain::AvatarRef::Webp))
    );
}

#[test]
fn unknown_actor_is_not_found() {
    let repository = Arc::new(FakeRepository::new());
    let actor = Actor::new(UserId::new(404), Role::Member);

    assert_eq!(
        block_on(
            SetDisplayNameCommandHandler::new(repository).handle(SetDisplayNameCommand {
                user_id: actor.user_id(),
                display_name: Some(DisplayName::parse("Ada").unwrap()),
            })
        )
        .unwrap_err(),
        ApplicationError::UserNotFound
    );
}
