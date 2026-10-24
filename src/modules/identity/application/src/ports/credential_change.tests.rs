use chrono::{TimeZone, Utc};
use identity_domain::PasswordHash;
use kernel::UserId;

use super::CredentialChange;

#[test]
fn permanent_change_has_no_expiry() {
    let change =
        CredentialChange::permanent(UserId::new(7), PasswordHash::from_stored("h".to_owned()));

    assert_eq!(change.user_id(), UserId::new(7));
    assert_eq!(
        change.password_hash(),
        &PasswordHash::from_stored("h".to_owned())
    );
    assert_eq!(change.expires_at(), None);
}

#[test]
fn temporary_change_carries_expiry() {
    let expires = Utc.with_ymd_and_hms(2026, 10, 24, 12, 0, 0).unwrap();
    let change = CredentialChange::temporary(
        UserId::new(3),
        PasswordHash::from_stored("t".to_owned()),
        expires,
    );

    assert_eq!(change.user_id(), UserId::new(3));
    assert_eq!(
        change.password_hash(),
        &PasswordHash::from_stored("t".to_owned())
    );
    assert_eq!(change.expires_at(), Some(expires));
}
