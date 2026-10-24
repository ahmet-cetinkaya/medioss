use identity_domain::{Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use std::sync::Arc;

use super::{SetUserStatusCommand, SetUserStatusCommandHandler};
use crate::ports::CommandHandler as _;
use crate::{
    ports::Clock as _,
    test_support::{block_on, FakeRepository, FixedClock},
    ApplicationError,
};

fn user(id: i64, role: Role, disabled: bool) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse("user@example.com").unwrap(),
        PasswordHash::from_stored("hash".to_owned()),
        role,
        FixedClock::new().now(),
    )
    .with_disabled(disabled)
}

#[test]
fn member_actor_is_forbidden() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member, false));
    let member = Actor::new(UserId::new(9), Role::Member);

    let result = block_on(set_user_status(&repository, member, UserId::new(2), true));

    assert_eq!(result, Err(ApplicationError::Forbidden));
}

#[test]
fn self_disable_is_refused() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(set_user_status(&repository, admin, UserId::new(1), true));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
}

#[test]
fn disabling_the_last_active_admin_is_refused() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, false));
    repository.seed_user(user(2, Role::Admin, true));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(set_user_status(&repository, admin, UserId::new(1), true));

    assert_eq!(result, Err(ApplicationError::LastActiveAdmin));
}

#[test]
fn admin_disables_a_member() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, false));
    repository.seed_user(user(2, Role::Member, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    block_on(set_user_status(&repository, admin, UserId::new(2), true)).unwrap();

    let disabled = repository
        .user_by_id(UserId::new(2))
        .map(|user| user.is_disabled())
        .unwrap_or(false);
    assert!(disabled);
}

#[test]
fn unknown_target_is_not_found() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(1, Role::Admin, false));
    let admin = Actor::new(UserId::new(1), Role::Admin);

    let result = block_on(set_user_status(&repository, admin, UserId::new(99), true));

    assert_eq!(result, Err(ApplicationError::UserNotFound));
}

async fn set_user_status(
    repository: &Arc<FakeRepository>,
    actor: kernel::Actor,
    target: UserId,
    disabled: bool,
) -> Result<(), ApplicationError> {
    SetUserStatusCommandHandler::new(repository.clone())
        .handle(SetUserStatusCommand {
            actor,
            target,
            disabled,
        })
        .await
}
