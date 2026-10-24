use super::*;

fn command(name: &str) -> RenameCategoryCommand {
    RenameCategoryCommand {
        id: 1,
        name: name.into(),
    }
}

#[test]
fn accepts_a_valid_name() {
    assert_eq!(
        RenameCategoryCommandValidator.validate(&command("Nature")),
        Ok(())
    );
}

#[test]
fn rejects_a_blank_name() {
    assert_eq!(
        RenameCategoryCommandValidator.validate(&command("   ")),
        Err(ApplicationError::InvalidInput("category name"))
    );
}

#[test]
fn rejects_a_name_over_the_length_limit() {
    assert_eq!(
        RenameCategoryCommandValidator.validate(&command(&"a".repeat(81))),
        Err(ApplicationError::InvalidInput("category name"))
    );
}
