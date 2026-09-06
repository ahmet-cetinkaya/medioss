use std::{fmt::Display, sync::Arc};

use tracing_subscriber::EnvFilter;
use web_api::{bootstrap::ensure_data_dir, config, router::router, state::AppState};

fn fail(err: impl Display) -> ! {
    eprintln!("{err}");
    std::process::exit(1)
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = config::from_env().unwrap_or_else(|err| fail(err));
    if let Err(err) = ensure_data_dir(&config.data_dir) {
        fail(err);
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
        eprintln!("server error: {err}");
        std::process::exit(1);
    }
}
