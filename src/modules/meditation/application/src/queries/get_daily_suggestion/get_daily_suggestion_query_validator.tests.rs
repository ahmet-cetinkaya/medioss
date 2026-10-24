use super::super::get_daily_suggestion_query::GetDailySuggestionQuery;
use super::GetDailySuggestionQueryValidator;

#[test]
fn accepts_the_query() {
    assert!(GetDailySuggestionQueryValidator
        .validate(&GetDailySuggestionQuery)
        .is_ok());
}
