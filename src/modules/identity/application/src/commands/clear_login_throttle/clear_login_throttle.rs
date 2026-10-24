/// Clears the failed-attempt record of one normalized identifier. The
/// operator CLI's `clear-throttle` subcommand is the only caller.
#[derive(Debug, Clone)]
pub struct ClearLoginThrottle {
    pub identifier: String,
}
