use std::{fmt::Display, sync::Arc};

use tracing_subscriber::EnvFilter;
use web_api::{
    bootstrap::{database::Database, ensure_data_dir, media_layout::migrate_media_layout},
    config::{self, Config},
    router::router,
    state::AppState,
};

mod operator;

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

fn main() {
    // Local convenience: load ./.env if present. Real environment variables
    // always win (dotenv_override is off), so production env files cannot
    // shadow an operator's explicit settings. A missing file is fine.
    let _ = dotenvy::dotenv();
    let cli = <Cli as clap::Parser>::parse();
    // Environment mutation is only sound while the process is single-threaded,
    // so the server config is read and published before the runtime exists.
    let server_config = cli.command.is_none().then(|| {
        let config = config::from_env().unwrap_or_else(|err| {
            eprintln!("{}", with_causes(&err));
            std::process::exit(1)
        });
        config::publish_dev_origin(&config);
        config
    });
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            eprintln!("could not start the async runtime: {err}");
            std::process::exit(1)
        })
        .block_on(run(cli, server_config));
}

async fn run(cli: Cli, server_config: Option<Config>) {
    match cli.command {
        Some(Commands::ResetPassword { email }) => {
            return operator::run_reset_password(&email).await;
        }
        Some(Commands::ClearThrottle { identifier }) => {
            return operator::run_clear_throttle(&identifier).await;
        }
        None => {}
    }
    let Some(config) = server_config else { return };

    init_tracing();
    config::warn_about_weakened_defaults(&config);
    if let Err(err) = ensure_data_dir(&config.data_dir) {
        fail(with_causes(&err));
    }
    migrate_media_layout(&config.data_dir);

    let listener = bind_listener(config.bind_host, config.port).await;
    let database = web_api::bootstrap::database::open(&config.data_dir.join("medioss.db"))
        .await
        .unwrap_or_else(|err| fail(with_causes(&err)));
    migrate_all(&database).await;
    seed_builtin_soundscapes(&database, &config).await;

    let state = build_state(config, database).await;
    if let Err(err) = axum::serve(listener, router(state)).await {
        fail(format!("server error: {err}"));
    }
}

fn init_tracing() {
    let filter = match std::env::var(EnvFilter::DEFAULT_ENV) {
        // The subscriber does not exist yet, so this one report cannot use tracing.
        Ok(directives) => EnvFilter::try_new(directives).unwrap_or_else(|err| {
            eprintln!("ignoring malformed RUST_LOG, defaulting to `info`: {err}");
            EnvFilter::new("info")
        }),
        Err(_) => EnvFilter::new("info"),
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn bind_listener(host: std::net::IpAddr, port: u16) -> tokio::net::TcpListener {
    let addr = std::net::SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| fail(format!("could not bind {addr}: {err}")));
    // Port 0 binds an ephemeral port, so only the bound address is informative.
    match listener.local_addr() {
        Ok(bound) => tracing::info!("medioss listening on {bound}"),
        Err(err) => tracing::warn!("medioss listening on {addr}, bound address unavailable: {err}"),
    }
    listener
}

async fn migrate_all(database: &Database) {
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
}

async fn seed_builtin_soundscapes(database: &Database, config: &Config) {
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
}

async fn build_state(config: Config, database: Database) -> AppState {
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(database.pool.clone());
    if let Err(err) = session_store.migrate().await {
        fail(format!("session store migration failed: {err}"));
    }
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();
    let origin = || Arc::new(config.medioss_origin.clone());
    let meditation = web_api::meditation::meditation_dependencies(
        database.connection.clone(),
        &config.data_dir,
        origin(),
    );
    let portability =
        web_api::portability::portability_dependencies(database.connection.clone(), origin());
    let mood_tracker =
        web_api::mood_tracker::mood_tracker_dependencies(database.connection.clone(), origin());
    let soundscape = web_api::soundscape::soundscape_dependencies(
        database.connection.clone(),
        &config.data_dir,
        origin(),
    );
    let deletion: Arc<dyn identity_presentation::UserDeletionListener> =
        Arc::new(web_api::user_deletion::CrossModuleUserDeletion::new(
            Arc::clone(&meditation.notes),
            Arc::clone(&meditation.playback),
            Arc::clone(&meditation.sessions),
            Arc::clone(&soundscape.mixes),
            Arc::clone(&mood_tracker.entries),
            config.data_dir.join("media").join("identity"),
        ));
    let avatar_store = Arc::new(web_api::avatar_store::FileAvatarStore::new(
        config.data_dir.join("media").join("identity"),
    ));
    AppState::new(
        Arc::new(config),
        Arc::new(session_store),
        meditation,
        mood_tracker,
        soundscape,
        portability,
        Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            database.connection,
        )),
        hasher,
        deletion,
        avatar_store,
    )
}
