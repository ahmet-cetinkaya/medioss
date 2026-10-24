use super::*;

#[test]
fn accepts_any_identifiers() {
    let command = AddMemberCommand {
        program_id: 1,
        meditation_id: 2,
    };
    assert_eq!(AddMemberCommandValidator.validate(&command), Ok(()));
}
