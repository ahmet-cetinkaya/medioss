use super::{UpdateProgramCommand, UpdateProgramCommandValidator};
use crate::ApplicationError;

fn command(title: &str) -> UpdateProgramCommand {
    UpdateProgramCommand {
        program_id: 1,
        title: title.into(),
        description: None,
    }
}

#[test]
fn accepts_a_title() {
    assert_eq!(
        UpdateProgramCommandValidator.validate(&command("Calm")),
        Ok(())
    );
}

#[test]
fn rejects_a_blank_title() {
    assert!(matches!(
        UpdateProgramCommandValidator.validate(&command("  ")),
        Err(ApplicationError::InvalidInput(_))
    ));
}
