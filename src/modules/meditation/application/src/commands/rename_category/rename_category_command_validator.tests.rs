use super::*;

fn command(name: &str) -> RenameCategoryCommand {
    RenameCategoryCommand {
        category_id: 1,
        name: name.into(),
    }
}

#[test]
fn accepts_a_nonempty_name() {
    assert_eq!(
        RenameCategoryCommandValidator.validate(&command("Morning")),
        Ok(())
    );
}

#[test]
fn rejects_an_empty_or_blank_name() {
    for name in ["", "   "] {
        assert_eq!(
            RenameCategoryCommandValidator.validate(&command(name)),
            Err(ApplicationError::InvalidInput("name must not be empty"))
        );
    }
}

#[test]
fn normalized_name_trims_surrounding_whitespace() {
    assert_eq!(
        RenameCategoryCommandValidator.normalized_name(&command("  Morning  ")),
        "Morning"
    );
}
