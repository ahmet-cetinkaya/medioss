use super::{SetupStatusQuery, SetupStatusQueryValidator};

#[test]
fn accepts_the_query() {
    assert!(SetupStatusQueryValidator
        .validate(&SetupStatusQuery)
        .is_ok());
}
