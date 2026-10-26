use super::{FavoriteMeditationCommand, FavoriteMeditationCommandValidator};
use crate::test_support::user_a;

#[test]
fn accepts_any_command_because_there_is_no_input_rule() {
    let command = FavoriteMeditationCommand {
        user_id: user_a(),
        meditation_id: 1,
    };

    assert!(FavoriteMeditationCommandValidator
        .validate(&command)
        .is_ok());
}
