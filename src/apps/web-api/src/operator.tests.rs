use identity_application::ports::{
    InitialAdminRegistration, LoginThrottleRepository, PasswordHasher, UserRepository,
};

use super::{
    generate_temporary_password, run_clear_throttle, run_reset_password, TEMP_PASSWORD_ALPHABET,
};

const TEMPORARY_PASSWORD_LENGTH: usize = 20;
const AMBIGUOUS_CHARACTERS: [char; 5] = ['0', 'O', '1', 'l', 'I'];
const ENV_VARS: [&str; 3] = ["MEDIOSS_DATA_DIR", "SESSION_SECRET", "MEDIOSS_ORIGIN"];
const OLD_PASSWORD: &str = "correct-horse";

#[test]
fn temporary_password_has_fixed_length() {
    assert_eq!(
        generate_temporary_password().chars().count(),
        TEMPORARY_PASSWORD_LENGTH
    );
}

#[test]
fn temporary_password_uses_only_the_alphabet() {
    for _ in 0..50 {
        let password = generate_temporary_password();
        assert!(
            password
                .bytes()
                .all(|byte| TEMP_PASSWORD_ALPHABET.contains(&byte)),
            "{password}"
        );
    }
}

#[test]
fn temporary_password_excludes_ambiguous_characters() {
    for _ in 0..50 {
        let password = generate_temporary_password();
        assert!(
            !password.contains(AMBIGUOUS_CHARACTERS),
            "ambiguous character in {password}"
        );
    }
}

#[test]
fn temporary_passwords_differ_between_calls() {
    assert_ne!(generate_temporary_password(), generate_temporary_password());
}

/// Removes the variables on drop so a failing assertion cannot leak them.
struct EnvVarsGuard;

impl Drop for EnvVarsGuard {
    fn drop(&mut self) {
        for name in ENV_VARS {
            std::env::remove_var(name);
        }
    }
}

// The only test that touches these variables, so it cannot race. clear-throttle
// reads the whole server configuration, hence three variables, not one.
#[tokio::test]
async fn reset_password_replaces_the_credential_and_clear_throttle_drops_the_attempts() {
    let directory = tempfile::tempdir().unwrap();
    let database = web_api::bootstrap::database::open(&directory.path().join("medioss.db"))
        .await
        .unwrap();
    identity_infrastructure::migrate(&database.connection)
        .await
        .unwrap();
    let repository = identity_infrastructure::SeaOrmUserRepository::new(database.connection);
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();
    let email = identity_domain::Email::parse("member@example.com").unwrap();
    let old_hash = hasher
        .hash(&identity_domain::RawPassword::parse(OLD_PASSWORD).unwrap())
        .await
        .unwrap();
    let seeded = repository
        .insert_initial_admin_if_uninitialized(InitialAdminRegistration::new(
            email.clone(),
            old_hash.clone(),
            chrono::Utc::now(),
        ))
        .await
        .unwrap();
    assert!(!seeded.must_change_password());

    let _guard = EnvVarsGuard;
    std::env::set_var("MEDIOSS_DATA_DIR", directory.path());
    std::env::set_var("SESSION_SECRET", "test-secret");
    std::env::set_var("MEDIOSS_ORIGIN", "https://medioss.example");

    repository
        .record_failed_attempt("member@example.com", chrono::Utc::now())
        .await
        .unwrap();
    run_reset_password("member@example.com").await;
    // Applying a credential change itself wipes the throttle row.
    assert!(repository
        .failed_attempts("member@example.com")
        .await
        .unwrap()
        .is_none());
    let reset = repository.find_by_email(&email).await.unwrap().unwrap();
    assert_ne!(reset.password_hash().as_str(), old_hash.as_str());
    assert!(reset.must_change_password());
    assert!(!hasher
        .verify(OLD_PASSWORD, reset.password_hash())
        .await
        .unwrap());

    repository
        .record_failed_attempt("member@example.com", chrono::Utc::now())
        .await
        .unwrap();
    assert!(repository
        .failed_attempts("member@example.com")
        .await
        .unwrap()
        .is_some());
    run_clear_throttle("  Member@Example.COM ").await;
    assert!(repository
        .failed_attempts("member@example.com")
        .await
        .unwrap()
        .is_none());
}
