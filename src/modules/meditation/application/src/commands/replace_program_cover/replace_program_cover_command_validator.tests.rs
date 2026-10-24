use kernel::UserId;

use super::{ReplaceProgramCoverCommand, ReplaceProgramCoverCommandValidator};

#[test]
fn accepts_any_command_because_there_is_no_input_rule() {
    let command = ReplaceProgramCoverCommand {
        requester_id: UserId::new(1),
        program_id: 1,
        bytes: Vec::new(),
    };

    assert!(ReplaceProgramCoverCommandValidator
        .validate(&command)
        .is_ok());
}
