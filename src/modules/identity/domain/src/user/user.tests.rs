use chrono::{TimeDelta, TimeZone, Utc};
use kernel::UserId;

use super::*;
use crate::user::{Email, IdentityError, PasswordHash, Role};

const HASH: &str = "stored-password-hash";

fn created_at() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 8, 10, 0, 0).unwrap()
}

fn email() -> Email {
    Email::parse("admin@example.com").unwrap()
}

fn password_hash() -> PasswordHash {
    PasswordHash::from_stored(HASH.to_owned())
}

#[test]
fn permanent_admin_credentials_do_not_expire_or_require_change() {
    let user = User::new_permanent(
        UserId::new(1),
        email(),
        password_hash(),
        Role::Admin,
        created_at(),
    );

    assert_eq!(user.id(), UserId::new(1));
    assert_eq!(user.email().as_str(), "admin@example.com");
    assert_eq!(user.password_hash().as_str(), HASH);
    assert_eq!(user.role(), Role::Admin);
    assert_eq!(user.created_at(), created_at());
    assert_eq!(user.auth_version(), 0);
    assert!(!user.must_change_password());
    assert_eq!(user.password_expires_at(), None);
}

#[test]
fn temporary_credentials_require_change_and_expire_after_24_hours() {
    let user = User::new_temporary(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
    )
    .unwrap();

    assert!(user.must_change_password());
    assert_eq!(
        user.password_expires_at(),
        Some(created_at() + TimeDelta::hours(24))
    );
}

#[test]
fn temporary_credentials_are_rejected_at_expiry_as_invalid_credentials() {
    let user = User::new_temporary(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
    )
    .unwrap();

    assert_eq!(
        user.ensure_password_is_active_at(created_at() + TimeDelta::hours(24)),
        Err(IdentityError::InvalidCredentials)
    );
}

#[test]
fn temporary_credentials_are_active_before_expiry() {
    let user = User::new_temporary(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
    )
    .unwrap();

    assert_eq!(
        user.ensure_password_is_active_at(
            created_at() + TimeDelta::hours(24) - TimeDelta::seconds(1)
        ),
        Ok(())
    );
}

#[test]
fn replacing_a_password_with_permanent_credentials_invalidates_sessions() {
    let user = User::new_temporary(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
    )
    .unwrap();
    let replacement = PasswordHash::from_stored("replacement-hash".to_owned());

    let changed = user.replace_with_permanent_password(replacement).unwrap();

    assert_eq!(changed.auth_version(), 1);
    assert!(!changed.must_change_password());
    assert_eq!(changed.password_expires_at(), None);
    assert_eq!(changed.password_hash().as_str(), "replacement-hash");
}

#[test]
fn replacing_a_password_with_temporary_credentials_sets_a_new_expiry() {
    let user = User::new_permanent(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
    );
    let reset_at = created_at() + TimeDelta::hours(2);
    let replacement = PasswordHash::from_stored("replacement-hash".to_owned());

    let changed = user
        .replace_with_temporary_password(replacement, reset_at)
        .unwrap();

    assert_eq!(changed.auth_version(), 1);
    assert!(changed.must_change_password());
    assert_eq!(
        changed.password_expires_at(),
        Some(reset_at + TimeDelta::hours(24))
    );
}

#[test]
fn restored_users_reject_inconsistent_credential_state() {
    let result = User::restore(
        UserId::new(2),
        email(),
        password_hash(),
        Role::Member,
        created_at(),
        3,
        false,
        Some(created_at() + TimeDelta::hours(24)),
        false,
        None,
        None,
    );

    assert_eq!(result, Err(IdentityError::InvalidCredentialState));
}

#[test]
fn a_user_maps_to_the_kernel_actor_role() {
    let user = User::new_permanent(
        UserId::new(42),
        email(),
        password_hash(),
        Role::Admin,
        created_at(),
    );

    let actor = user.actor();

    assert_eq!(actor.user_id(), UserId::new(42));
    assert_eq!(actor.role(), Role::Admin);
}

#[test]
fn user_debug_does_not_contain_email_or_password_hash() {
    let user = User::new_permanent(
        UserId::new(1),
        email(),
        password_hash(),
        Role::Admin,
        created_at(),
    );
    let debug = format!("{user:?}");

    assert!(!debug.contains("admin@example.com"));
    assert!(!debug.contains(HASH));
}
