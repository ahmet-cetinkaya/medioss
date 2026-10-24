use super::ListLibraryQueryValidator;
use crate::queries::list_library::ListLibraryQuery;

fn query(text: Option<&str>) -> ListLibraryQuery {
    ListLibraryQuery {
        query: text.map(str::to_owned),
    }
}

#[test]
fn accepts_any_query() {
    assert!(ListLibraryQueryValidator
        .validate(&query(Some("sleep")))
        .is_ok());
}

#[test]
fn absent_search_is_not_a_filter() {
    assert_eq!(
        ListLibraryQueryValidator.non_blank_search(&query(None)),
        None
    );
}

#[test]
fn blank_search_is_not_a_filter() {
    assert_eq!(
        ListLibraryQueryValidator.non_blank_search(&query(Some("  \t"))),
        None
    );
}

#[test]
fn non_blank_search_is_kept_as_typed() {
    assert_eq!(
        ListLibraryQueryValidator.non_blank_search(&query(Some(" Sleep "))),
        Some(" Sleep ")
    );
}
