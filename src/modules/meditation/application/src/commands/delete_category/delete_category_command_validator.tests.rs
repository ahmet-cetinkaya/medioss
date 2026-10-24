use super::*;

#[test]
fn accepts_any_identifiers() {
    let command = DeleteCategoryCommand { category_id: 1 };
    assert_eq!(DeleteCategoryCommandValidator.validate(&command), Ok(()));
}
