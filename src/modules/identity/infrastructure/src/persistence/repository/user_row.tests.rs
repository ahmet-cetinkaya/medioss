use chrono::{TimeZone, Utc};
use identity_domain::{AvatarRef, DisplayName, Email, PasswordHash, Role, User};
use kernel::UserId;

use super::{insert_model, is_unique_violation, restore_user, role_from_str, role_to_str};
use crate::persistence::users;

fn user(role: Role) -> User {
    User::new_permanent(
        UserId::new(5),
        Email::parse("row@medioss.test").unwrap(),
        PasswordHash::from_stored("hash-5".to_owned()),
        role,
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
    )
}

#[test]
fn roles_round_trip_through_their_storage_strings() {
    assert_eq!(role_to_str(Role::Admin), "admin");
    assert_eq!(role_to_str(Role::Member), "member");
    assert_eq!(role_from_str("admin"), Ok(Role::Admin));
    assert_eq!(role_from_str("member"), Ok(Role::Member));
    assert_eq!(role_from_str("superuser"), Err(identity_application::ports::RepositoryError::OperationFailed));
}

#[test]
fn restore_user_maps_every_column() {
    let restored = restore_user(users::Model {
        id: 5,
        email: "row@medioss.test".to_owned(),
        password_hash: "hash-5".to_owned(),
        role: "admin".to_owned(),
        must_change_password: true,
        auth_version: 3,
        password_expires_at: Some("2026-09-26T15:00:00+00:00".to_owned()),
        disabled: true,
        display_name: Some("Ahmet".to_owned()),
        avatar_ext: Some("webp".to_owned()),
        avatar_key: Some("avatar-0123456789abcdef.webp".to_owned()),
        created_at: "2026-09-26T12:00:00+00:00".to_owned(),
    })
    .unwrap();

    assert_eq!(restored.id(), UserId::new(5));
    assert_eq!(restored.email().as_str(), "row@medioss.test");
    assert_eq!(restored.role(), Role::Admin);
    assert_eq!(restored.auth_version(), 3);
    assert!(restored.must_change_password());
    assert!(restored.is_disabled());
    assert_eq!(
        restored.password_expires_at(),
        Some(Utc.with_ymd_and_hms(2026, 9, 26, 15, 0, 0).unwrap())
    );
    assert_eq!(restored.display_name().map(|name| name.as_str()), Some("Ahmet"));
    assert_eq!(
        restored.avatar().map(|avatar| avatar.key().to_owned()),
        Some("avatar-0123456789abcdef.webp".to_owned())
    );
    assert_eq!(
        restored.created_at(),
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap()
    );
}

#[test]
fn restore_user_rejects_inconsistent_credential_state() {
    // must_change_password is true but no expiry is stored: the row is
    // inconsistent and must fail rather than silently lose the expiry.
    let model = users::Model {
        id: 6,
        email: "broken@medioss.test".to_owned(),
        password_hash: "hash-6".to_owned(),
        role: "member".to_owned(),
        must_change_password: true,
        auth_version: 0,
        password_expires_at: None,
        disabled: false,
        display_name: None,
        avatar_ext: None,
        avatar_key: None,
        created_at: "2026-09-26T12:00:00+00:00".to_owned(),
    };

    assert!(restore_user(model).is_err());
}

#[test]
fn insert_model_maps_user_fields_back() {
    let with_avatar = user(Role::Member)
        .with_display_name(Some(DisplayName::parse("Deniz").unwrap()))
        .with_avatar(Some(AvatarRef::from_extension("webp").unwrap()));

    let model = insert_model(&with_avatar, "row@medioss.test");

    assert_eq!(model.email.unwrap(), "row@medioss.test");
    assert_eq!(model.password_hash.unwrap(), "hash-5");
    assert_eq!(model.role.unwrap(), "member");
    assert!(!model.must_change_password.unwrap());
    assert_eq!(model.auth_version.unwrap(), 0);
    assert_eq!(model.password_expires_at.unwrap(), None);
    assert_eq!(model.display_name.unwrap(), Some("Deniz".to_owned()));
    assert_eq!(model.avatar_ext.unwrap(), Some("webp".to_owned()));
    assert!(model.avatar_key.unwrap().expect("avatar key").ends_with(".webp"));
    assert_eq!(model.created_at.unwrap(), "2026-09-26T12:00:00+00:00");
}

#[test]
fn unique_violation_is_detected_by_message() {
    let unique = sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
        "UNIQUE constraint failed: users.email".to_owned(),
    ));
    let other = sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
        "FOREIGN KEY constraint failed".to_owned(),
    ));

    assert!(is_unique_violation(&unique));
    assert!(!is_unique_violation(&other));
    assert!(!is_unique_violation(&sea_orm::DbErr::RecordNotFound(
        "users".to_owned()
    )));
}
