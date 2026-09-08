use identity_domain::required_delay;

use super::clear_login_throttle;
use crate::test_support::{
    block_on, email, password, FakeHasher, FakeLoginAdmission, FakeRepository, FixedClock,
};

#[test]
fn cli_recovery_can_bypass_and_clear_normalized_login_throttle() {
    let repository = FakeRepository::new();
    repository.set_failed_attempts(email().as_str(), 100);
    let mixed_case_email = format!("  {}  ", email().as_str().to_uppercase());

    block_on(clear_login_throttle(&repository, &mixed_case_email)).unwrap();

    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}

#[test]
fn cli_recovery_can_bypass_throttle_before_clearing_it() {
    let repository = FakeRepository::with_permanent_admin();
    let hasher = FakeHasher::new();
    let clock = FixedClock::new();
    let admission = FakeLoginAdmission::saturated(required_delay(1));
    let command = super::super::Authenticate::new(
        &repository,
        &hasher,
        &clock,
        &admission,
        FakeHasher::hash_for(&password()),
    );
    repository.set_failed_attempts(email().as_str(), 100);

    let user = block_on(command.execute_bypassing_throttle(email().as_str(), password().as_str()))
        .unwrap();

    assert_eq!(user.email(), &email());
    assert_eq!(hasher.verify_calls(), 1);
    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}
