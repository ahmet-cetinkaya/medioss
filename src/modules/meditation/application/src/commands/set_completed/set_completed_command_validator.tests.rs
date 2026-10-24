use kernel::UserId;

use super::{SetCompletedCommand, SetCompletedCommandValidator};

#[test]
fn accepts_both_flag_values_because_there_is_no_input_rule() {
    for completed in [true, false] {
        let command = SetCompletedCommand {
            user_id: UserId::new(1),
            meditation_id: 1,
            completed,
        };

        assert!(SetCompletedCommandValidator.validate(&command).is_ok());
    }
}
