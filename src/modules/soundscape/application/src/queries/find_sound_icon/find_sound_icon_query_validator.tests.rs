use super::*;

#[test]
fn accepts_the_query() {
    assert_eq!(
        FindSoundIconQueryValidator.validate(&FindSoundIconQuery { id: 1 }),
        Ok(())
    );
}
