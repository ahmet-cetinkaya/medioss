use std::{fmt::Display, sync::Arc};

use tracing_subscriber::EnvFilter;
use web_api::{bootstrap::ensure_data_dir, config, router::router, state::AppState};

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
    tracing::info!("medioss listening on {addr}");

    let state = AppState {
        config: Arc::new(config),
    };
    if let Err(err) = axum::serve(listener, router(state)).await {
        fail(format!("server error: {err}"));
    }
}
