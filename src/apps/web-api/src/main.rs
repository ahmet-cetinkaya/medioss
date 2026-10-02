use std::{fmt::Display, sync::Arc};

use tracing_subscriber::EnvFilter;
use web_api::{bootstrap::ensure_data_dir, config, router::router};

fn fail(err: impl Display) -> ! {
    tracing::error!("{err}");
    std::process::exit(1)
}

// Each `Display` renders only its own message, so the `#[source]` chain has to
// be appended here to keep the underlying cause visible exactly once.
fn with_causes(err: &dyn std::error::Error) -> String {
    std::iter::successors(err.source(), |cause| cause.source())
        .fold(err.to_string(), |message, cause| {
            format!("{message}: {cause}")
        })
}

#[derive(clap::Parser)]
#[command(name = "medioss", about = "MediOSS server and operator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Reset an account password from the host, without a running server.
    #[command(name = "reset-password")]
    ResetPassword { email: String },
    /// Clear the login throttle for an identifier from the host.
    #[command(name = "clear-throttle")]
    ClearThrottle { identifier: String },
}

#[tokio::main]
async fn main() {
    // Local convenience: load ./.env if present. Real environment variables
    // always win (dotenv_override is off), so production env files cannot
    // shadow an operator's explicit settings. A missing file is fine.
    let _ = dotenvy::dotenv();
    let cli = <Cli as clap::Parser>::parse();
    match cli.command {
        Some(Commands::ResetPassword { email }) => {
            return run_reset_password(&email).await;
        }
        Some(Commands::ClearThrottle { identifier }) => {
            return run_clear_throttle(&identifier).await;
        }
        None => {}
    }

    let filter = match std::env::var(EnvFilter::DEFAULT_ENV) {
        // The subscriber does not exist yet, so this one report cannot use tracing.
        Ok(directives) => EnvFilter::try_new(directives).unwrap_or_else(|err| {
            eprintln!("ignoring malformed RUST_LOG, defaulting to `info`: {err}");
            EnvFilter::new("info")
        }),
        Err(_) => EnvFilter::new("info"),
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = config::from_env().unwrap_or_else(|err| fail(with_causes(&err)));
    if let Err(err) = ensure_data_dir(&config.data_dir) {
        fail(with_causes(&err));
    }

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| fail(format!("could not bind {addr}: {err}")));
    // Port 0 binds an ephemeral port, so only the bound address is informative.
    match listener.local_addr() {
        Ok(bound) => tracing::info!("medioss listening on {bound}"),
        Err(err) => tracing::warn!("medioss listening on {addr}, bound address unavailable: {err}"),
    }

    let database_path = config.data_dir.join("medioss.db");
    let database = web_api::bootstrap::database::open(&database_path)
        .await
        .unwrap_or_else(|err| fail(with_causes(&err)));
    if let Err(err) = identity_infrastructure::migrate(&database.connection).await {
        fail(format!("identity migration failed: {err}"));
    }
    if let Err(err) = meditation_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("meditation migration failed: {err}"));
    }
    if let Err(err) = soundscape_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("soundscape migration failed: {err}"));
    }
    if let Err(err) = mood_tracker_infrastructure::migrations::migrate(&database.connection).await {
        fail(format!("mood tracker migration failed: {err}"));
    }
    match soundscape_infrastructure::builtin::seed_if_present(
        &database.connection,
        &config.data_dir.join("media"),
    )
    .await
    {
        Ok(true) => tracing::info!("built-in soundscape set seeded"),
        Ok(false) => tracing::debug!("no built-in soundscape set found; skipping seed"),
        Err(err) => tracing::warn!("built-in soundscape seed failed: {err}"),
    }
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(database.pool.clone());
    if let Err(err) = session_store.migrate().await {
        fail(format!("session store migration failed: {err}"));
    }
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();
    let meditation = web_api::meditation::meditation_dependencies(
        database.connection.clone(),
        &config.data_dir,
        std::sync::Arc::new(config.medioss_origin.clone()),
    );
    let mood_tracker = web_api::mood_tracker::mood_tracker_dependencies(
        database.connection.clone(),
        std::sync::Arc::new(config.medioss_origin.clone()),
    );
    let soundscape = web_api::soundscape::soundscape_dependencies(
        database.connection.clone(),
        &config.data_dir,
        std::sync::Arc::new(config.medioss_origin.clone()),
    );
    let session_store = std::sync::Arc::new(session_store);
    let deletion: std::sync::Arc<dyn identity_presentation::UserDeletionListener> =
        std::sync::Arc::new(web_api::user_deletion::CrossModuleUserDeletion::new(
            std::sync::Arc::clone(&session_store),
            std::sync::Arc::clone(&meditation.notes),
            std::sync::Arc::clone(&meditation.playback),
            std::sync::Arc::clone(&meditation.sessions),
            std::sync::Arc::clone(&soundscape.mixes),
            std::sync::Arc::clone(&mood_tracker.entries),
            config.data_dir.join("media"),
        ));
    let state = web_api::state::AppState::new(
        Arc::new(config),
        session_store,
        meditation,
        mood_tracker,
        soundscape,
        std::sync::Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            database.connection,
        )),
        hasher,
        deletion,
    );
    if let Err(err) = axum::serve(listener, router(state)).await {
        fail(format!("server error: {err}"));
    }
}

async fn run_reset_password(email: &str) {
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

async fn run_clear_throttle(identifier: &str) {
    let config = config::from_env().unwrap_or_else(|err| fail(with_causes(&err)));
    let database_path = config.data_dir.join("medioss.db");
    let database = web_api::bootstrap::database::open(&database_path)
        .await
        .unwrap_or_else(|err| fail(with_causes(&err)));

    let repository = identity_infrastructure::SeaOrmUserRepository::new(database.connection);
    match identity_application::commands::clear_login_throttle(&repository, identifier).await {
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
