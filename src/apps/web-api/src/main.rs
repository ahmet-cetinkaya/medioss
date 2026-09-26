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

#[tokio::main]
async fn main() {
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
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(database.pool.clone());
    if let Err(err) = session_store.migrate().await {
        fail(format!("session store migration failed: {err}"));
    }
    let hasher = identity_infrastructure::BoundedArgon2PasswordHasher::new();
    let state = web_api::state::AppState::new(
        Arc::new(config),
        std::sync::Arc::new(session_store),
        std::sync::Arc::new(identity_infrastructure::SeaOrmUserRepository::new(
            database.connection,
        )),
        hasher,
    );
    if let Err(err) = axum::serve(listener, router(state)).await {
        fail(format!("server error: {err}"));
    }
}
