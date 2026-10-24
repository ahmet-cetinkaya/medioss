use super::{SetSoundIconCommand, SetSoundIconCommandValidator};

#[test]
fn accepts_any_bytes_because_the_normalizer_owns_icon_rules() {
    let command = SetSoundIconCommand {
        id: 1,
        bytes: vec![1, 2, 3],
    };

    assert!(SetSoundIconCommandValidator.validate(&command).is_ok());
}
