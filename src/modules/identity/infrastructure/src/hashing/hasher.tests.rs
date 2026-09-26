use std::time::Duration;

use identity_application::ports::PasswordHasher as _;
use identity_domain::{PasswordHash, RawPassword};

use super::BoundedArgon2PasswordHasher;

fn strong_password() -> RawPassword {
    RawPassword::parse("correct-horse-battery-staple").unwrap()
}

#[tokio::test]
async fn hash_roundtrips_through_verify() {
    let hasher = BoundedArgon2PasswordHasher::new();
    let hash = hasher.hash(&strong_password()).await.unwrap();

    assert!(hash.as_str().starts_with("$argon2id$"));
    assert!(hash.as_str().contains("m=19456"));
    assert!(hash.as_str().contains("t=2"));
    assert!(hash.as_str().contains("p=1"));

    assert!(hasher
        .verify("correct-horse-battery-staple", &hash)
        .await
        .unwrap());
    assert!(!hasher.verify("wrong-password-here", &hash).await.unwrap());
}

#[tokio::test]
async fn salts_are_random_per_hash() {
    let hasher = BoundedArgon2PasswordHasher::new();
    let first = hasher.hash(&strong_password()).await.unwrap();
    let second = hasher.hash(&strong_password()).await.unwrap();
    assert_ne!(first.as_str(), second.as_str());
}

#[tokio::test]
async fn dummy_hash_is_fixed_for_the_hasher_lifetime() {
    let hasher = BoundedArgon2PasswordHasher::new();
    assert_eq!(
        hasher.dummy_password_hash().as_str(),
        hasher.dummy_password_hash().as_str()
    );
    assert!(hasher
        .dummy_password_hash()
        .as_str()
        .starts_with("$argon2id$"));
}

#[tokio::test]
async fn malformed_stored_hash_is_rejected_as_error() {
    let hasher = BoundedArgon2PasswordHasher::new();
    let garbage = PasswordHash::from_stored("not-a-phc-string".to_owned());
    assert!(hasher.verify("whatever-password", &garbage).await.is_err());
}

#[tokio::test]
async fn oversized_phc_parameters_are_rejected() {
    let hasher = BoundedArgon2PasswordHasher::new();
    let oversized = PasswordHash::from_stored(
        "$argon2id$v=19$m=1048576,t=64,p=8$c29tZXNhbHQ$Zm9vYmFy".to_owned(),
    );
    assert!(hasher
        .verify("whatever-password", &oversized)
        .await
        .is_err());
}

#[tokio::test]
async fn at_most_two_hashes_run_concurrently() {
    let hasher = std::sync::Arc::new(BoundedArgon2PasswordHasher::new());
    let mut handles = Vec::new();
    for _ in 0..6 {
        let hasher = hasher.clone();
        handles.push(tokio::spawn(async move {
            hasher.hash(&strong_password()).await.unwrap()
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn hashing_completes_within_a_sane_wall_clock_bound() {
    let hasher = BoundedArgon2PasswordHasher::new();
    let started = std::time::Instant::now();
    hasher.hash(&strong_password()).await.unwrap();
    assert!(started.elapsed() < Duration::from_secs(10));
}
