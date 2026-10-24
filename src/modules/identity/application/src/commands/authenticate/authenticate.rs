/// Verifies an identifier and password. The identifier is normalized
/// before any throttle or hash work; `bypass_throttle` is the operator
/// recovery path (`execute_bypassing_throttle` at HEAD) and is not set by
/// the HTTP surface.
#[derive(Debug, Clone)]
pub struct Authenticate {
    pub identifier: String,
    pub password: String,
    pub bypass_throttle: bool,
}
