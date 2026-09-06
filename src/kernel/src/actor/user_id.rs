/// Identifies a user. The field is private so M1 can add validation to `new`
/// without touching read sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserId(i64);

impl UserId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}
