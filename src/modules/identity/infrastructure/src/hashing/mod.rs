use std::time::Duration;

use argon2::{
    password_hash::{
        rand_core::OsRng as ArgonOsRng, PasswordHash as PhcHash, PasswordHasher as _,
        PasswordVerifier as _, SaltString,
    },
    Argon2, Params,
};
use async_trait::async_trait;
use identity_application::ports::{PasswordHasher, PasswordHasherError};
use identity_domain::{PasswordHash, RawPassword};

use tokio::sync::Semaphore;

const M_COST: u32 = 19 * 1024;
const T_COST: u32 = 2;
const P_COST: u32 = 1;
const OUTPUT_LEN: usize = 32;

const PERMIT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct BoundedArgon2PasswordHasher {
    permits: Semaphore,
    dummy_hash: PasswordHash,
}

impl BoundedArgon2PasswordHasher {
    pub fn new() -> Self {
        Self {
            permits: Semaphore::new(2),
            dummy_hash: PasswordHash::from_stored(dummy_phc()),
        }
    }

    pub fn dummy_password_hash(&self) -> PasswordHash {
        self.dummy_hash.clone()
    }

    fn configured_params() -> Params {
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("configured Argon2id parameters are always valid")
    }

    fn production_max_params() -> Params {
        Params::new(M_COST * 2, T_COST * 4, P_COST * 2, Some(OUTPUT_LEN))
            .expect("production maxima are always valid")
    }
}

impl Default for BoundedArgon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

fn dummy_phc() -> String {
    let salt = SaltString::generate(&mut ArgonOsRng);
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        BoundedArgon2PasswordHasher::configured_params(),
    )
    .hash_password(b"medioss-dummy-verification-target", &salt)
    .expect("dummy hash with valid parameters always succeeds")
    .to_string()
}

async fn bounded<T, F>(permits: &Semaphore, work: F) -> Result<T, PasswordHasherError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let _guard = tokio::time::timeout(PERMIT_TIMEOUT, permits.acquire())
        .await
        .map_err(|_| PasswordHasherError)?
        .map_err(|_| PasswordHasherError)?;
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| PasswordHasherError)
}

#[async_trait]
impl PasswordHasher for BoundedArgon2PasswordHasher {
    async fn hash(&self, password: &RawPassword) -> Result<PasswordHash, PasswordHasherError> {
        let password = password.as_str().to_owned();
        bounded(&self.permits, move || {
            let salt = SaltString::generate(&mut ArgonOsRng);
            let argon = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                BoundedArgon2PasswordHasher::configured_params(),
            );
            let phc = argon2::PasswordHasher::hash_password(
                &argon,
                password.as_bytes(),
                &salt,
            )
            .map_err(|_| PasswordHasherError)?;
            Ok(PasswordHash::from_stored(phc.to_string()))
        })
        .await
        .and_then(|inner| inner)
    }

    async fn verify(
        &self,
        password: &str,
        password_hash: &PasswordHash,
    ) -> Result<bool, PasswordHasherError> {
        let password = password.to_owned();
        let stored = password_hash.as_str().to_owned();
        bounded(&self.permits, move || {
            let parsed = PhcHash::new(&stored).map_err(|_| PasswordHasherError)?;
            let maxima = BoundedArgon2PasswordHasher::production_max_params();
            let params = argon2::Params::try_from(&parsed).map_err(|_| PasswordHasherError)?;
            if params.m_cost() > maxima.m_cost()
                || params.t_cost() > maxima.t_cost()
                || params.p_cost() > maxima.p_cost()
            {
                return Err(PasswordHasherError);
            }
            let argon = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                BoundedArgon2PasswordHasher::configured_params(),
            );
            match argon.verify_password(password.as_bytes(), &parsed) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .and_then(|inner| inner)
    }
}

#[cfg(test)]
#[path = "hasher.tests.rs"]
mod tests;
