use chrono::{TimeZone, Utc};
use identity_domain::{Email, PasswordHash};

use super::InitialAdminRegistration;

#[test]
fn carries_email_hash_and_creation_time() {
    let created_at = Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap();
    let registration = InitialAdminRegistration::new(
        Email::parse("admin@medioss.test").unwrap(),
        PasswordHash::from_stored("hash-1".to_owned()),
        created_at,
    );

    assert_eq!(registration.email(), &Email::parse("admin@medioss.test").unwrap());
    assert_eq!(registration.password_hash(), &PasswordHash::from_stored("hash-1".to_owned()));
    assert_eq!(registration.created_at(), created_at);
}
