use std::fmt::Display;

/// Logs the error and exits with status 1. The exit itself cannot be unit-tested.
pub fn fail(err: impl Display) -> ! {
    tracing::error!("{err}");
    std::process::exit(1)
}

/// Prints the error to stderr and exits with status 1. Use it before
/// `init_tracing` runs, when `tracing::error!` would go nowhere. The exit
/// itself cannot be unit-tested; `tests/startup_failure_e2e.rs` covers it.
pub fn exit_before_logging(err: &dyn std::error::Error) -> ! {
    eprintln!("{}", with_causes(err));
    std::process::exit(1)
}

// Each `Display` renders only its own message, so the `#[source]` chain has to
// be appended here to keep the underlying cause visible exactly once.
pub fn with_causes(err: &dyn std::error::Error) -> String {
    std::iter::successors(err.source(), |cause| cause.source())
        .fold(err.to_string(), |message, cause| {
            format!("{message}: {cause}")
        })
}

#[cfg(test)]
#[path = "report.tests.rs"]
mod tests;
