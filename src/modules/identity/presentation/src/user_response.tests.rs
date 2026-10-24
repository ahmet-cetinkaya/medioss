use chrono::{TimeZone, Utc};
use identity_domain::{Email, PasswordHash, Role, User};
use kernel::UserId;

use super::{role_to_str, user_response};

fn user(role: Role) -> User {
    User::new_permanent(
        UserId::new(11),
        Email::parse("shown@medioss.test").unwrap(),
        PasswordHash::from_stored("hash-11".to_owned()),
        role,
        Utc.with_ymd_and_hms(2026, 9, 26, 12, 0, 0).unwrap(),
    )
}

#[test]
fn role_names_are_the_wire_values() {
    assert_eq!(role_to_str(Role::Admin), "admin");
    assert_eq!(role_to_str(Role::Member), "member");
}

#[test]
fn user_response_never_exposes_the_password_hash() {
    let response = user_response(
        &user(Role::Admin)
            .with_display_name(Some(identity_domain::DisplayName::parse("Ahmet").unwrap())),
    );

    assert_eq!(response.id, 11);
    assert_eq!(response.email, "shown@medioss.test");
    assert_eq!(response.role, "admin");
    assert_eq!(response.display_name, Some("Ahmet".to_owned()));
    assert!(!response.must_change_password);
    assert!(!response.has_avatar);
    assert!(!serde_json::to_string(&response)
        .unwrap()
        .contains("hash-11"));
}

#[test]
fn member_permanent_user_reports_no_forced_password_change() {
    let response = user_response(&user(Role::Member));

    assert_eq!(response.role, "member");
    assert!(!response.must_change_password);
}
