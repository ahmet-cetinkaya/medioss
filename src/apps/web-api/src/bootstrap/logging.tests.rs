use std::env::VarError;

use super::filter_from_directives;

#[test]
fn valid_directives_are_kept() {
    let filter = filter_from_directives(Ok("web_api=debug".to_owned()));

    assert_eq!(filter.to_string(), "web_api=debug");
}

#[test]
fn malformed_directives_fall_back_to_info() {
    let filter = filter_from_directives(Ok("web_api=notalevel".to_owned()));

    assert_eq!(filter.to_string(), "info");
}

#[test]
fn unset_variable_defaults_to_info() {
    let filter = filter_from_directives(Err(VarError::NotPresent));

    assert_eq!(filter.to_string(), "info");
}
