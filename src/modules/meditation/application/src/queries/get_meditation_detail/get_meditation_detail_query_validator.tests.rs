use super::super::get_meditation_detail_query::GetMeditationDetailQuery;
use super::GetMeditationDetailQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetMeditationDetailQueryValidator
        .validate(&GetMeditationDetailQuery { meditation_id: 1 })
        .is_ok());
}
