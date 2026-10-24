use super::{DeleteNoteCommand, DeleteNoteCommandValidator};

#[test]
fn accepts_any_command_because_there_is_no_input_rule() {
    let command = DeleteNoteCommand {
        note_id: 1,
        meditation_id: 2,
        user_id: kernel::UserId::new(3),
    };

    assert!(DeleteNoteCommandValidator.validate(&command).is_ok());
}
