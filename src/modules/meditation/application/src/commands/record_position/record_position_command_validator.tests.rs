use kernel::UserId;

use super::super::record_position_command::MAX_POSITION_MS;
use super::{RecordPositionCommand, RecordPositionCommandValidator};
use crate::ApplicationError;

fn command_at(position_ms: i64) -> RecordPositionCommand {
    RecordPositionCommand {
        user_id: UserId::new(1),
        meditation_id: 1,
        position_ms,
    }
}

#[test]
fn accepts_positions_at_both_bounds() {
    assert!(RecordPositionCommandValidator
        .validate(&command_at(0))
        .is_ok());
    assert!(RecordPositionCommandValidator
        .validate(&command_at(MAX_POSITION_MS))
        .is_ok());
}

#[test]
fn rejects_negative_position() {
    assert!(matches!(
        RecordPositionCommandValidator.validate(&command_at(-1)),
        Err(ApplicationError::InvalidInput(_))
    ));
}

#[test]
fn rejects_position_beyond_the_cap() {
    assert!(matches!(
        RecordPositionCommandValidator.validate(&command_at(MAX_POSITION_MS + 1)),
        Err(ApplicationError::InvalidInput(_))
    ));
}
