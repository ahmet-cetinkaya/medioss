use super::*;

#[test]
fn accepts_any_id() {
    let command = DeleteCategoryCommand { id: 1 };
    assert_eq!(DeleteCategoryCommandValidator.validate(&command), Ok(()));
}
