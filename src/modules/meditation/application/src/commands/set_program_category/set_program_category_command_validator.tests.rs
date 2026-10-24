use super::*;

#[test]
fn accepts_any_identifiers() {
    let command = SetProgramCategoryCommand {
        program_id: 1,
        category_id: None,
    };
    assert_eq!(
        SetProgramCategoryCommandValidator.validate(&command),
        Ok(())
    );
}
