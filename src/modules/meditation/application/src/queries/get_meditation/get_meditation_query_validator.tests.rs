use super::super::get_meditation_query::GetMeditationQuery;
use super::GetMeditationQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetMeditationQueryValidator
        .validate(&GetMeditationQuery { meditation_id: 1 })
        .is_ok());
}
