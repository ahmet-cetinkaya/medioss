use super::*;
use kernel::UserId;

#[test]
fn accepts_any_id() {
    let command = DeleteMixCommand {
        id: 1,
        user_id: UserId::new(7),
    };
    assert_eq!(DeleteMixCommandValidator.validate(&command), Ok(()));
}
