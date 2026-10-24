use super::*;

#[test]
fn builds_a_category_with_trimmed_name_and_slug() {
    let category = SoundCategory::new(7, "  Nature ", " nature ").unwrap();

    assert_eq!(
        category,
        SoundCategory {
            id: 7,
            name: "Nature".to_owned(),
            slug: "nature".to_owned(),
        }
    );
}

#[test]
fn rejects_a_blank_name() {
    assert_eq!(
        SoundCategory::new(1, " ", "nature"),
        Err(DomainError::InvalidName)
    );
}

#[test]
fn rejects_a_blank_slug() {
    assert_eq!(
        SoundCategory::new(1, "Nature", ""),
        Err(DomainError::InvalidName)
    );
}
