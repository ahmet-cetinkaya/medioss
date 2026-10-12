use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{clear_avatar, read_avatar, replace_avatar};
use crate::ports::Clock;
use crate::ports::{
    AvatarStore, AvatarStoreError, CredentialChange, InitialAdminRegistration, MemberRegistration,
    RepositoryError, UserRepository,
};
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
    next_key: std::sync::atomic::AtomicU64,
    fail_writes: std::sync::atomic::AtomicBool,
}

impl CountingAvatarStore {
    fn accepting() -> Self {
        Self {
            written: std::sync::Mutex::new(Vec::new()),
            removed: std::sync::Mutex::new(Vec::new()),
            normalize_result: std::sync::Mutex::new(Ok((vec![1, 2, 3], "webp"))),
            next_key: std::sync::atomic::AtomicU64::new(1),
            fail_writes: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn rejecting() -> Self {
        Self::rejecting_with(AvatarStoreError::InvalidPayload)
    }

    fn rejecting_with(error: AvatarStoreError) -> Self {
        Self {
            normalize_result: std::sync::Mutex::new(Err(error)),
            ..Self::accepting()
        }
    }

    fn failing_writes() -> Self {
        Self {
            fail_writes: std::sync::atomic::AtomicBool::new(true),
            ..Self::accepting()
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
        extension: &'static str,
        _bytes: &[u8],
    ) -> Result<AvatarRef, AvatarStoreError> {
        if self.fail_writes.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AvatarStoreError::Storage);
        }
        let sequence = self
            .next_key
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let avatar =
            AvatarRef::new(format!("avatar-{sequence:016x}.{extension}"), extension).unwrap();
        self.written
            .lock()
            .unwrap()
            .push((id.value(), avatar.extension()));
        Ok(avatar)
    }

    async fn read(&self, _id: UserId, _avatar: &AvatarRef) -> Result<Vec<u8>, AvatarStoreError> {
        Ok(vec![9, 9, 9])
    }

    async fn remove(&self, id: UserId, avatar: &AvatarRef) -> Result<(), AvatarStoreError> {
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

/// Delegates to a fake repository but fails every `set_profile` call, to
/// exercise the avatar replacement rollback path.
struct FailingProfileRepository<F>(F);

#[async_trait::async_trait]
impl<F: UserRepository> UserRepository for FailingProfileRepository<F> {
    async fn count(&self) -> Result<u64, RepositoryError> {
        self.0.count().await
    }

    async fn list(&self) -> Result<Vec<User>, RepositoryError> {
        self.0.list().await
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        self.0.find_by_email(email).await
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        self.0.find_by_id(id).await
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: InitialAdminRegistration,
    ) -> Result<User, RepositoryError> {
        self.0
            .insert_initial_admin_if_uninitialized(registration)
            .await
    }

    async fn insert_member(
        &self,
        registration: MemberRegistration,
    ) -> Result<User, RepositoryError> {
        self.0.insert_member(registration).await
    }

    async fn apply_credential_change(
        &self,
        credential_change: CredentialChange,
    ) -> Result<User, RepositoryError> {
        self.0.apply_credential_change(credential_change).await
    }

    async fn set_disabled(&self, id: UserId, disabled: bool) -> Result<(), RepositoryError> {
        self.0.set_disabled(id, disabled).await
    }

    async fn set_profile(
        &self,
        id: UserId,
        display_name: Option<DisplayName>,
        _avatar: Option<AvatarRef>,
    ) -> Result<User, RepositoryError> {
        let _ = (id, display_name);
        Err(RepositoryError::OperationFailed)
    }

    async fn delete_user(&self, id: UserId) -> Result<bool, RepositoryError> {
        self.0.delete_user(id).await
    }

    async fn count_active_admins(&self) -> Result<u64, RepositoryError> {
        self.0.count_active_admins().await
    }

    async fn delete_login_attempts(&self, normalized_email: &str) -> Result<(), RepositoryError> {
        self.0.delete_login_attempts(normalized_email).await
    }
}

#[test]
fn replace_persists_and_unlinks_the_previous_file() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap();

    let avatar = updated.avatar().unwrap();
    assert_eq!(avatar.extension(), "webp");
    assert_eq!(avatar.key(), "avatar-0000000000000001.webp");
    assert_eq!(*store.written.lock().unwrap(), vec![(2, "webp")]);
    assert_eq!(*store.removed.lock().unwrap(), vec![(2, "png")]);
}

#[test]
fn replace_stages_a_new_key_and_only_deletes_the_old_one_after_the_update() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let previous_key = repository
        .user_by_id(UserId::new(2))
        .unwrap()
        .avatar()
        .unwrap()
        .key()
        .to_owned();
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap();

    let new_avatar = updated.avatar().unwrap();
    assert_eq!(new_avatar.key(), "avatar-0000000000000001.webp");
    assert_ne!(new_avatar.key(), previous_key);
    assert_eq!(
        repository
            .user_by_id(UserId::new(2))
            .unwrap()
            .avatar()
            .unwrap()
            .key(),
        "avatar-0000000000000001.webp"
    );
    assert_eq!(*store.removed.lock().unwrap(), vec![(2, "png")]);
}

#[test]
fn replace_on_db_update_failure_keeps_the_old_avatar_and_the_staged_blob_so_a_committed_update_never_dangles(
) {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let repository = FailingProfileRepository(repository);
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap_err();

    assert_eq!(error, ApplicationError::Repository);
    let stored = repository.0.user_by_id(UserId::new(2)).unwrap();
    assert_eq!(stored.avatar().unwrap().key(), "avatar.png");
    let written = store.written.lock().unwrap().clone();
    assert_eq!(written, vec![(2, "webp")]);
    let removed = store.removed.lock().unwrap().clone();
    // set_profile may have committed before failing, so nothing is removed:
    // an orphaned blob is cheaper than a dangling reference.
    assert!(removed.is_empty());
}

#[test]
fn replace_on_store_write_failure_keeps_everything_untouched() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let store = CountingAvatarStore::failing_writes();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap_err();

    assert_eq!(error, ApplicationError::Repository);
    assert_eq!(
        repository
            .user_by_id(UserId::new(2))
            .unwrap()
            .avatar()
            .unwrap()
            .key(),
        "avatar.png"
    );
    assert!(store.removed.lock().unwrap().is_empty());
}

#[test]
fn first_upload_writes_the_avatar_without_deleting_anything() {
    let repository = FakeRepository::new();
    repository.seed_user(user(2, Role::Member));
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap();

    assert_eq!(
        updated.avatar().unwrap().key(),
        "avatar-0000000000000001.webp"
    );
    assert_eq!(
        repository
            .user_by_id(UserId::new(2))
            .unwrap()
            .avatar()
            .unwrap()
            .key(),
        "avatar-0000000000000001.webp"
    );
    assert_eq!(*store.written.lock().unwrap(), vec![(2, "webp")]);
    assert!(store.removed.lock().unwrap().is_empty());
}

#[test]
fn replace_and_clear_keep_the_display_name() {
    let repository = FakeRepository::new();
    let named = user(2, Role::Member).with_display_name(Some(
        identity_domain::DisplayName::parse("Ahmet QA").unwrap(),
    ));
    repository.seed_user(named);
    store_avatar(&repository, "webp");
    let store = CountingAvatarStore::accepting();
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(replace_avatar(&repository, &store, &actor, vec![7])).unwrap();
    assert_eq!(updated.display_name().map(|n| n.as_str()), Some("Ahmet QA"));

    block_on(clear_avatar(&repository, &store, &actor)).unwrap();
    let cleared = repository.user_by_id(UserId::new(2)).unwrap();
    assert_eq!(cleared.display_name().map(|n| n.as_str()), Some("Ahmet QA"));
    assert_eq!(cleared.avatar(), None);
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
fn replace_maps_normalization_failures_by_who_is_at_fault() {
    // Client-input problems keep 4xx semantics; server-side worker trouble
    // becomes operational errors (500/503 at the API edge).
    let cases = [
        (
            AvatarStoreError::InvalidPayload,
            ApplicationError::InvalidInput(identity_domain::IdentityError::InvalidProfile),
        ),
        (
            AvatarStoreError::OutputTooLarge,
            ApplicationError::PayloadTooLarge,
        ),
        (AvatarStoreError::WorkerFailed, ApplicationError::Repository),
        (
            AvatarStoreError::Busy,
            ApplicationError::ProcessingUnavailable,
        ),
        (AvatarStoreError::Storage, ApplicationError::Repository),
    ];
    for (store_error, expected) in cases {
        let repository = FakeRepository::new();
        repository.seed_user(user(2, Role::Member));
        let store = CountingAvatarStore::rejecting_with(store_error);
        let actor = Actor::new(UserId::new(2), Role::Member);

        let error = block_on(replace_avatar(&repository, &store, &actor, vec![1])).unwrap_err();

        assert_eq!(error, expected, "{store_error:?}");
        assert!(store.written.lock().unwrap().is_empty());
        assert!(store.removed.lock().unwrap().is_empty());
    }
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
