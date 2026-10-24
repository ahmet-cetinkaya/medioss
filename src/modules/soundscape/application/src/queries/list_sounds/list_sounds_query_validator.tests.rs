use super::*;

#[test]
fn accepts_the_query() {
    assert_eq!(ListSoundsQueryValidator.validate(&ListSoundsQuery), Ok(()));
}
