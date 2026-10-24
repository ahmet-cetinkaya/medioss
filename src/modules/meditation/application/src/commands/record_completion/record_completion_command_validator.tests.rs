use chrono::{Duration, Utc};
use kernel::UserId;

use super::{RecordCompletionCommand, RecordCompletionCommandValidator};
use crate::ApplicationError;

fn command_lasting(duration: Duration) -> RecordCompletionCommand {
    let started_at = Utc::now();
    RecordCompletionCommand {
        user_id: UserId::new(1),
        meditation_id: 1,
        started_at,
        finished_at: started_at + duration,
    }
}

#[test]
fn accepts_finish_after_start_and_equal_to_start() {
    for duration in [Duration::minutes(10), Duration::zero()] {
        assert!(RecordCompletionCommandValidator
            .validate(&command_lasting(duration))
            .is_ok());
    }
}

#[test]
fn rejects_finish_before_start() {
    assert!(matches!(
        RecordCompletionCommandValidator.validate(&command_lasting(Duration::seconds(-1))),
        Err(ApplicationError::InvalidInput(_))
    ));
}
