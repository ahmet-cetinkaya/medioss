use std::sync::Arc;

use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role, User};
use kernel::{Actor, UserId};

use super::{ReplaceAvatar, ReplaceAvatarHandler, ReplaceAvatarService};
use crate::{
    ports::{AvatarStoreError, Clock as _, UserRepository},
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

/// Delegates to a fake repository but fails every `set_profile` call, to
/// exercise the avatar replacement rollback path.
struct FailingProfileRepository {
    inner: Arc<FakeRepository>,
}

#[async_trait::async_trait]
impl UserRepository for FailingProfileRepository {
    async fn count(&self) -> Result<u64, crate::ports::RepositoryError> {
        self.inner.count().await
    }

    async fn list(&self) -> Result<Vec<User>, crate::ports::RepositoryError> {
        self.inner.list().await
    }

    async fn find_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<User>, crate::ports::RepositoryError> {
        self.inner.find_by_email(email).await
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, crate::ports::RepositoryError> {
        self.inner.find_by_id(id).await
    }

    async fn insert_initial_admin_if_uninitialized(
        &self,
        registration: crate::ports::InitialAdminRegistration,
    ) -> Result<User, crate::ports::RepositoryError> {
        self.inner
            .insert_initial_admin_if_uninitialized(registration)
            .await
    }

    async fn insert_member(
        &self,
        registration: crate::ports::MemberRegistration,
    ) -> Result<User, crate::ports::RepositoryError> {
        self.inner.insert_member(registration).await
    }

    async fn apply_credential_change(
        &self,
        credential_change: crate::ports::CredentialChange,
    ) -> Result<User, crate::ports::RepositoryError> {
        self.inner.apply_credential_change(credential_change).await
    }

    async fn set_disabled(
        &self,
        id: UserId,
        disabled: bool,
    ) -> Result<(), crate::ports::RepositoryError> {
        self.inner.set_disabled(id, disabled).await
    }

    async fn set_profile(
        &self,
        _id: UserId,
        _display_name: Option<DisplayName>,
        _avatar: Option<AvatarRef>,
    ) -> Result<User, crate::ports::RepositoryError> {
        Err(crate::ports::RepositoryError::OperationFailed)
    }

    async fn delete_user(&self, id: UserId) -> Result<bool, crate::ports::RepositoryError> {
        self.inner.delete_user(id).await
    }

    async fn count_active_admins(&self) -> Result<u64, crate::ports::RepositoryError> {
        self.inner.count_active_admins().await
    }

    async fn delete_login_attempts(
        &self,
        normalized_email: &str,
    ) -> Result<(), crate::ports::RepositoryError> {
        self.inner.delete_login_attempts(normalized_email).await
    }
}

#[async_trait::async_trait]
impl crate::ports::LoginThrottleRepository for FailingProfileRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<identity_domain::FailedAttempts>, crate::ports::RepositoryError> {
        crate::ports::LoginThrottleRepository::failed_attempts(
            self.inner.as_ref(),
            normalized_identifier,
        )
        .await
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<identity_domain::FailedAttempts, crate::ports::RepositoryError> {
        crate::ports::LoginThrottleRepository::record_failed_attempt(
            self.inner.as_ref(),
            normalized_identifier,
            attempted_at,
        )
        .await
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), crate::ports::RepositoryError> {
        crate::ports::LoginThrottleRepository::clear_failed_attempts(
            self.inner.as_ref(),
            normalized_identifier,
        )
        .await
    }
}

#[test]
fn replace_persists_and_unlinks_the_previous_file() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap();

    let avatar = updated.avatar().unwrap();
    assert_eq!(avatar.extension(), "webp");
    assert_eq!(avatar.key(), "avatar-0000000000000001.webp");
    assert_eq!(store.written(), vec![(2, "webp")]);
    assert_eq!(store.removed(), vec![(2, "png")]);
}

#[test]
fn replace_stages_a_new_key_and_only_deletes_the_old_one_after_the_update() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let previous_key = repository
        .user_by_id(UserId::new(2))
        .unwrap()
        .avatar()
        .unwrap()
        .key()
        .to_owned();
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap();

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
    assert_eq!(store.removed(), vec![(2, "png")]);
}

#[test]
fn replace_on_db_update_failure_keeps_the_old_avatar_and_the_staged_blob_so_a_committed_update_never_dangles(
) {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let failing = Arc::new(FailingProfileRepository {
        inner: repository.clone(),
    });
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(
        ReplaceAvatarService::new(failing, store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap_err();

    assert_eq!(error, ApplicationError::Repository);
    let stored = repository.user_by_id(UserId::new(2)).unwrap();
    assert_eq!(stored.avatar().unwrap().key(), "avatar.png");
    assert_eq!(store.written(), vec![(2, "webp")]);
    // set_profile may have committed before failing, so nothing is removed:
    // an orphaned blob is cheaper than a dangling reference.
    assert!(store.removed().is_empty());
}

#[test]
fn replace_on_store_write_failure_keeps_everything_untouched() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    store_avatar(&repository, "png");
    let store = Arc::new(FakeAvatarStore::failing_writes());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap_err();

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
    assert!(store.removed().is_empty());
}

#[test]
fn first_upload_writes_the_avatar_without_deleting_anything() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap();

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
    assert_eq!(store.written(), vec![(2, "webp")]);
    assert!(store.removed().is_empty());
}

#[test]
fn replace_keeps_the_display_name() {
    let repository = Arc::new(FakeRepository::new());
    let named = user(2, Role::Member).with_display_name(Some(
        identity_domain::DisplayName::parse("Ahmet QA").unwrap(),
    ));
    repository.seed_user(named);
    store_avatar(&repository, "webp");
    let store = Arc::new(FakeAvatarStore::accepting());
    let actor = Actor::new(UserId::new(2), Role::Member);

    let updated = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![7],
        }),
    )
    .unwrap();
    assert_eq!(updated.display_name().map(|n| n.as_str()), Some("Ahmet QA"));
}

#[test]
fn replace_rejects_invalid_payloads_and_writes_nothing() {
    let repository = Arc::new(FakeRepository::new());
    repository.seed_user(user(2, Role::Member));
    let store = Arc::new(FakeAvatarStore::rejecting_with(
        AvatarStoreError::InvalidPayload,
    ));
    let actor = Actor::new(UserId::new(2), Role::Member);

    let error = block_on(
        ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
            actor,
            bytes: vec![0],
        }),
    )
    .unwrap_err();

    assert_eq!(error, ApplicationError::UnsupportedMedia);
    assert!(store.written().is_empty());
    assert!(store.removed().is_empty());
}

#[test]
fn replace_maps_normalization_failures_by_who_is_at_fault() {
    // Client-input problems keep 4xx semantics; server-side worker trouble
    // becomes operational errors (500/503 at the API edge).
    let cases = [
        (
            AvatarStoreError::InvalidPayload,
            ApplicationError::UnsupportedMedia,
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
        let repository = Arc::new(FakeRepository::new());
        repository.seed_user(user(2, Role::Member));
        let store = Arc::new(FakeAvatarStore::rejecting_with(store_error));
        let actor = Actor::new(UserId::new(2), Role::Member);

        let error = block_on(
            ReplaceAvatarService::new(repository.clone(), store.clone()).handle(ReplaceAvatar {
                actor,
                bytes: vec![1],
            }),
        )
        .unwrap_err();

        assert_eq!(error, expected, "{store_error:?}");
        assert!(store.written().is_empty());
        assert!(store.removed().is_empty());
    }
}
