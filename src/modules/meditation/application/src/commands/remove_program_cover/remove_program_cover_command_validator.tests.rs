use super::{RemoveProgramCoverCommand, RemoveProgramCoverCommandValidator};

#[test]
fn any_program_id_is_valid_because_existence_is_the_handlers_check() {
    let command = RemoveProgramCoverCommand { program_id: 1 };
    assert!(RemoveProgramCoverCommandValidator
        .validate(&command)
        .is_ok());
}
