use super::*;

const MAX_EMAIL_LENGTH: usize = 254;

#[test]
fn parse_trims_and_lowercases_an_email() {
    let email = Email::parse("  Admin@Example.COM  ").unwrap();

    assert_eq!(email.as_str(), "admin@example.com");
}

#[test]
fn parse_accepts_an_email_at_the_length_limit() {
    let email = Email::parse(format!("{}@b", "a".repeat(MAX_EMAIL_LENGTH - 2))).unwrap();

    assert_eq!(email.as_str().len(), MAX_EMAIL_LENGTH);
}

#[test]
fn parse_rejects_an_empty_email() {
    assert_eq!(Email::parse("   "), Err(IdentityError::InvalidEmail));
}

#[test]
fn parse_rejects_an_email_without_an_at_sign() {
    assert_eq!(
        Email::parse("admin.example.com"),
        Err(IdentityError::InvalidEmail)
    );
}

#[test]
fn parse_rejects_an_email_over_the_length_limit() {
    let input = format!("{}@b", "a".repeat(MAX_EMAIL_LENGTH - 1));

    assert_eq!(Email::parse(input), Err(IdentityError::InvalidEmail));
}

#[test]
fn manual_qa_displays_the_canonical_email() {
    let email = Email::parse("  Admin@Example.COM  ").unwrap();

    println!("canonical_email={}", email.as_str());
    assert_eq!(email.as_str(), "admin@example.com");
}
