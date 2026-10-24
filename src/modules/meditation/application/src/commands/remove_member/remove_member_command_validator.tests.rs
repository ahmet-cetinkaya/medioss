use super::*;

#[test]
fn accepts_any_identifiers() {
    let command = RemoveMemberCommand {
        program_id: 1,
        meditation_id: 2,
    };
    assert_eq!(RemoveMemberCommandValidator.validate(&command), Ok(()));
}
