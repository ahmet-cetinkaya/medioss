use identity_domain::{AvatarRef, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{clear_avatar, read_avatar, replace_avatar};
use crate::ports::Clock;
use crate::ports::{AvatarStore, AvatarStoreError};
use crate::test_support::{block_on, FakeRepository, FixedClock};
use crate::ApplicationError;

fn user(id: i64, role: Role) -> User {
    User::new_permanent(
        UserId::new(id),
        Email::parse(format!("user{id}@example.com")).unwrap(),
        PasswordHash::from_stored("hash".to_owned()),
        role,
        FixedClock::new().now(),
    )
}

struct CountingAvatarStore {
    written: std::sync::Mutex<Vec<(i64, &'static str)>>,
    removed: std::sync::Mutex<Vec<(i64, &'static str)>>,
    normalize_result: std::sync::Mutex<Result<(Vec<u8>, &'static str), AvatarStoreError>>,
}

impl CountingAvatarStore {
    fn accepting() -> Self {
        Self {
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            normalize_result: std::sync::Mutex::new(Ok((vec![1, 2, 3], "webp"))),
        }
    }

    fn rejecting() -> Self {
        Self {
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            normalize_result: std::sync::Mutex::new(Err(AvatarStoreError::InvalidPayload)),
        }
    }
}

#[async_trait::async_trait]
impl AvatarStore for CountingAvatarStore {
    async fn normalize(
        &self,
        _bytes: Vec<u8>,
    ) -> Result<(Vec<u8>, &'static str), AvatarStoreError> {
        self.normalize_result.lock().unwrap().clone()
    }

    async fn write(
        &self,
        id: UserId,
        avatar: AvatarRef,
        _bytes: &[u8],
    ) -> Result<(), AvatarStoreError> {
        self.written
            .lock()
            .unwrap()
            .push((id.value(), avatar.extension()));
        Ok(())
    }

    async fn read(&self, _id: UserId, _avatar: AvatarRef) -> Result<Vec<u8>, AvatarStoreError> {
        Ok(vec![9, 9, 9])
    }

    async fn remove(&self, id: UserId, avatar: AvatarRef) -> Result<(), AvatarStoreError> {
        self.removed
            .lock()
            .unwrap()
            .push((id.value(), avatar.extension()));
        Ok(())
    }
}

fn store_avatar(repository: &FakeRepository, ext: &str) {
    let changed = repository
        .user_by_id(UserId::new(2))
        .map(|user| user.with_avatar(Some(AvatarRef::from_extension(ext).unwrap())))
        .unwrap();
    repository.seed_user(changed);
}

#[test]
fn replace_persists_and_unlinks_the_previous_file() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap();

    assert_eq!(updated.avatar(), Some(AvatarRef::Webp));
    assert_eq!(*store.written.lock().unwrap(), vec![(2, "webp")]);
    assert_eq!(*store.removed.lock().unwrap(), vec![(2, "png")]);
}

#[test]
fn replace_rejects_invalid_payloads_and_writes_nothing() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    let store = CountingAvatarStore::rejecting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(replace_avatar(&repository, &store, &actor, vec![0])).unwrap_err();

    assert_eq!(
        error,
        ApplicationError::InvalidInput(identity_domain::IdentityError::InvalidProfile)
    );
    assert!(store.written.lock().unwrap().is_empty());
    assert!(store.removed.lock().unwrap().is_empty());
}

#[test]
fn clear_is_idempotent_and_removes_only_existing_files() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "webp");
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    block_on(clear_avatar(&repository, &store, &actor)).unwrap();
    block_on(clear_avatar(&repository, &store, &actor)).unwrap();

    assert_eq!(*store.removed.lock().unwrap(), vec![(2, "webp")]);
}

#[test]
fn read_allows_owner_and_admin_but_not_other_members() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    repository.seed_user(user(1, Role::Admin));
    repository.seed_user(user(3, Role::Member));
    store_avatar(&repository, "webp");
    let store = CountingAvatarStore::accepting();

    let owner = Actor::new(UserId::new(2), Role::Member);
    let admin = Actor::new(UserId::new(1), Role::Admin);
    let other = Actor::new(UserId::new(3), Role::Member);

    assert!(block_on(read_avatar(&repository, &store, UserId::new(2), &owner)).is_ok());
    assert!(block_on(read_avatar(&repository, &store, UserId::new(2), &admin)).is_ok());
    assert_eq!(
        block_on(read_avatar(&repository, &store, UserId::new(2), &other)).unwrap_err(),
        ApplicationError::Forbidden
    );
}

#[test]
fn read_of_a_user_without_an_avatar_is_not_found() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    let store = CountingAvatarStore::accepting();
    let owner = Actor::new(UserId::new(2), Role::Member);

    assert_eq!(
        block_on(read_avatar(&repository, &store, UserId::new(2), &owner)).unwrap_err(),
        ApplicationError::UserNotFound
    );
}
