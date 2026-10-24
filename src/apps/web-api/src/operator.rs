use web_api::config;

use crate::{fail, with_causes};

pub(crate) async fn run_reset_password(email: &str) {
    #[cfg(unix)]
    set_umask(0o077);

    let data_dir = match std::env::var("MEDIOSS_DATA_DIR") {
        Ok(value) if !value.trim().is_empty() => std::path::PathBuf::from(value),
        _ => {
            eprintln!("MEDIOSS_DATA_DIR is required");
            std::process::exit(1);
        }
    };
    let database_path = data_dir.join("medioss.db");
    if !database_path.exists() {
        eprintln!("no database found at {}", database_path.display());
        std::process::exit(1);
    }

    let database = match web_api::bootstrap::database::open(&database_path).await {
        Ok(database) => database,
        Err(err) => {
            eprintln!("could not open database: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = identity_infrastructure::migrate(&database.connection).await {
        eprintln!("migration failed: {err}");
        std::process::exit(1);
    }

    let repository = identity_infrastructure::SeaOrmUserRepository::new(database.connection);
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();

    let parsed_email = match identity_domain::Email::parse(email) {
        Ok(email) => email,
        Err(_) => {
            eprintln!("'{email}' is not a valid email address");
            std::process::exit(1);
        }
    };
    let user = match identity_application::ports::UserRepository::find_by_email(
        &repository,
        &parsed_email,
    )
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            eprintln!("no such user: {email}");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("database read failed: {}", with_causes(&err));
            std::process::exit(1);
        }
    };

    let temporary_password = generate_temporary_password();
    let raw = match identity_domain::RawPassword::parse(&temporary_password) {
        Ok(raw) => raw,
        Err(_) => {
            eprintln!("generated password rejected by policy");
            std::process::exit(1);
        }
    };
    let expires_at = chrono::Utc::now() + chrono::TimeDelta::hours(24);
    let password_hash = match identity_application::ports::PasswordHasher::hash(&hasher, &raw).await
    {
        Ok(hash) => hash,
        Err(err) => {
            eprintln!("password hashing failed: {}", with_causes(&err));
            std::process::exit(1);
        }
    };
    let credential_change = identity_application::ports::CredentialChange::temporary(
        user.id(),
        password_hash,
        expires_at,
    );
    match identity_application::ports::UserRepository::apply_credential_change(
        &repository,
        credential_change,
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            eprintln!("password reset failed: {}", with_causes(&err));
            std::process::exit(1);
        }
    }

    eprintln!("Temporary password (shown once; terminal scrollback may retain it):");
    println!("{temporary_password}");
}

const TEMP_PASSWORD_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

fn generate_temporary_password() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..20)
        .map(|_| {
            let index = rng.random_range(0..TEMP_PASSWORD_ALPHABET.len());
            TEMP_PASSWORD_ALPHABET[index] as char
        })
        .collect()
}

pub(crate) async fn run_clear_throttle(identifier: &str) {
    let config = config::from_env().unwrap_or_else(|err| fail(with_causes(&err)));
    let database_path = config.data_dir.join("medioss.db");
    let database = web_api::bootstrap::database::open(&database_path)
        .await
        .unwrap_or_else(|err| fail(with_causes(&err)));

    let repository: std::sync::Arc<dyn identity_application::ports::LoginThrottleRepository> =
        std::sync::Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            database.connection,
        ));
    use identity_application::commands::clear_login_throttle::ClearLoginThrottleHandler;
    let service =
        identity_application::commands::clear_login_throttle::ClearLoginThrottleService::new(
            repository,
        );
    match service
        .handle(
            identity_application::commands::clear_login_throttle::ClearLoginThrottle {
                identifier: identifier.to_owned(),
            },
        )
        .await
    {
        Ok(()) => eprintln!("throttle cleared for {identifier}"),
        Err(err) => fail(format!("clear-throttle failed: {err}")),
    }
}

#[cfg(unix)]
fn set_umask(mask: u32) {
    unsafe {
        libc::umask(mask);
    }
}

#[cfg(test)]
#[path = "operator.tests.rs"]
mod tests;
