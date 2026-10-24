use super::*;

fn command(name: &str) -> CreateCategoryCommand {
    CreateCategoryCommand { name: name.into() }
}

#[test]
fn accepts_a_nonempty_name() {
    assert_eq!(
        CreateCategoryCommandValidator.validate(&command("Morning")),
        Ok(())
    );
}

#[test]
fn rejects_an_empty_or_blank_name() {
    for name in ["", "   "] {
        assert_eq!(
            CreateCategoryCommandValidator.validate(&command(name)),
            Err(ApplicationError::InvalidInput("name must not be empty"))
        );
    }
}

#[test]
fn normalized_name_trims_surrounding_whitespace() {
    assert_eq!(
        CreateCategoryCommandValidator.normalized_name(&command("  Morning  ")),
        "Morning"
    );
}
