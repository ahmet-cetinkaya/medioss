use std::sync::Arc;

use identity_domain::required_delay;

use super::{ClearLoginThrottle, ClearLoginThrottleHandler, ClearLoginThrottleService};
use crate::commands::authenticate::AuthenticateHandler;
use crate::test_support::{
    block_on, email, password, FakeHasher, FakeLoginAdmission, FakeRepository, FixedClock,
};

#[test]
fn cli_recovery_clears_the_normalized_login_throttle() {
    let repository = Arc::new(FakeRepository::new());
    repository.set_failed_attempts(email().as_str(), 100);
    let mixed_case_email = format!("  {}  ", email().as_str().to_uppercase());

    block_on(
        ClearLoginThrottleService::new(repository.clone()).handle(ClearLoginThrottle {
            identifier: mixed_case_email,
        }),
    )
    .unwrap();

    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}

#[test]
fn cli_recovery_can_bypass_throttle_before_clearing_it() {
    let repository = Arc::new(FakeRepository::with_permanent_admin());
    let hasher = Arc::new(FakeHasher::new());
    let clock = Arc::new(FixedClock::new());
    let admission = Arc::new(FakeLoginAdmission::saturated(required_delay(1)));
    // The bypass path is authentication itself, not throttle clearing; this
    // preserves the HEAD regression coverage for `execute_bypassing_throttle`.
    use crate::commands::authenticate::{Authenticate, AuthenticateService};
    let command = AuthenticateService::new(
        repository.clone(),
        repository.clone(),
        hasher,
        clock,
        admission,
        FakeHasher::hash_for(&password()),
    );
    repository.set_failed_attempts(email().as_str(), 100);

    let user = block_on(command.handle(Authenticate {
        identifier: email().as_str().to_owned(),
        password: password().as_str().to_owned(),
        bypass_throttle: true,
    }))
    .unwrap();

    assert_eq!(user.email(), &email());
    assert_eq!(repository.failed_attempts(email().as_str()), 0);
}
