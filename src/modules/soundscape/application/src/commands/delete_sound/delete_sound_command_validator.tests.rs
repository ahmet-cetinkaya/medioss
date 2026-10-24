use super::{DeleteSoundCommand, DeleteSoundCommandValidator};

#[test]
fn accepts_any_id_because_there_is_no_input_rule() {
    let command = DeleteSoundCommand { id: 1 };

    assert!(DeleteSoundCommandValidator.validate(&command).is_ok());
}
