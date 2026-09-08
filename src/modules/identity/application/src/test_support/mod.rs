mod async_executor;
mod fake_hasher;
mod fake_repository;
mod fake_repository_state;
mod fixed_clock;

use identity_domain::{Email, RawPassword};

pub(crate) use async_executor::block_on;
pub(crate) use fake_hasher::FakeHasher;
pub(crate) use fake_repository::FakeRepository;
pub(crate) use fixed_clock::FixedClock;

pub(crate) fn email() -> Email {
    Email::parse("admin@local.invalid").unwrap()
}

pub(crate) fn password() -> RawPassword {
    RawPassword::parse("correct-password").unwrap()
}
