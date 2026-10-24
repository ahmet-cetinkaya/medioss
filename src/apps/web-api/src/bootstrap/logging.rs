use tracing_subscriber::EnvFilter;

const DEFAULT_DIRECTIVES: &str = "info";

/// Builds the log filter from the `RUST_LOG` lookup result; unset or malformed
/// values fall back to `info`.
pub fn filter_from_directives(env_value: Result<String, std::env::VarError>) -> EnvFilter {
    match env_value {
        // The subscriber does not exist yet, so this one report cannot use tracing.
        Ok(directives) => EnvFilter::try_new(directives).unwrap_or_else(|err| {
            eprintln!("ignoring malformed RUST_LOG, defaulting to `info`: {err}");
            EnvFilter::new(DEFAULT_DIRECTIVES)
        }),
        Err(_) => EnvFilter::new(DEFAULT_DIRECTIVES),
    }
}

/// Installs the global subscriber; panics if one is already installed.
pub fn init_tracing() {
    let filter = filter_from_directives(std::env::var(EnvFilter::DEFAULT_ENV));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[cfg(test)]
#[path = "logging.tests.rs"]
mod tests;
