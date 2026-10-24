use super::{DeleteProgramCommand, DeleteProgramCommandValidator};

#[test]
fn accepts_any_command_because_there_is_no_input_rule() {
    let command = DeleteProgramCommand { program_id: 1 };

    assert!(DeleteProgramCommandValidator.validate(&command).is_ok());
}
