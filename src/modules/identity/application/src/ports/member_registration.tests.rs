use chrono::{TimeZone, Utc};
use identity_domain::{Email, PasswordHash};

use super::MemberRegistration;

#[test]
fn carries_email_hash_and_creation_time() {
    let created_at = Utc.with_ymd_and_hms(2026, 10, 1, 8, 30, 0).unwrap();
    let registration = MemberRegistration::new(
        Email::parse("member@medioss.test").unwrap(),
        PasswordHash::from_stored("hash-2".to_owned()),
        created_at,
    );

    assert_eq!(
        registration.email(),
        &Email::parse("member@medioss.test").unwrap()
    );
    assert_eq!(
        registration.password_hash(),
        &PasswordHash::from_stored("hash-2".to_owned())
    );
    assert_eq!(registration.created_at(), created_at);
}
