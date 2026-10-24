use super::{DeleteMeditationCommand, DeleteMeditationCommandValidator};

#[test]
fn accepts_any_meditation_id_because_there_is_no_input_rule() {
    let command = DeleteMeditationCommand { meditation_id: 1 };

    assert!(DeleteMeditationCommandValidator.validate(&command).is_ok());
}
