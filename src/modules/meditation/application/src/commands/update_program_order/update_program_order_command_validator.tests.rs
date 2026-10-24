use super::{UpdateProgramOrderCommand, UpdateProgramOrderCommandValidator};

#[test]
fn accepts_any_command_because_there_is_no_input_rule() {
    let command = UpdateProgramOrderCommand {
        program_id: 1,
        meditation_ids: vec![3, 2, 1],
    };

    assert!(UpdateProgramOrderCommandValidator
        .validate(&command)
        .is_ok());
}
