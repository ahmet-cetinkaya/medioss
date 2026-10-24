use super::{ClearSoundIconCommand, ClearSoundIconCommandValidator};

#[test]
fn accepts_any_id_because_there_is_no_input_rule() {
    let command = ClearSoundIconCommand { id: 1 };

    assert!(ClearSoundIconCommandValidator.validate(&command).is_ok());
}
