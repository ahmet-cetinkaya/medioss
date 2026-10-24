use super::{parse_range, ByteRange, Unsatisfiable};

fn is_range(range: &Option<ByteRange>, start: u64, end_inclusive: u64) -> bool {
    matches!(range, Some(parsed)
        if parsed.start == start && parsed.end_inclusive == end_inclusive)
}

#[test]
fn an_absent_header_serves_the_full_body() {
    assert!(parse_range("", 100).unwrap().is_none());
}

#[test]
fn a_non_bytes_header_serves_the_full_body() {
    assert!(parse_range("items=0-5", 100).unwrap().is_none());
}

#[test]
fn a_multi_range_header_serves_the_full_body() {
    assert!(parse_range("bytes=0-5,10-15", 100).unwrap().is_none());
}

#[test]
fn a_malformed_spec_serves_the_full_body() {
    assert!(parse_range("bytes=abc", 100).unwrap().is_none());
    assert!(parse_range("bytes=-", 100).unwrap().is_none());
    assert!(parse_range("bytes=5-x", 100).unwrap().is_none());
    assert!(parse_range("bytes=x-5", 100).unwrap().is_none());
}

#[test]
fn a_start_after_the_end_is_unsatisfiable() {
    assert!(matches!(
        parse_range("bytes=19-10", 100),
        Err(Unsatisfiable)
    ));
}

#[test]
fn a_full_start_end_range_is_served_truncated_to_the_total() {
    assert!(is_range(&parse_range("bytes=10-19", 100).unwrap(), 10, 19));
    // An end past the total is clamped to the last byte.
    assert!(is_range(&parse_range("bytes=90-999", 100).unwrap(), 90, 99));
}

#[test]
fn a_start_at_or_past_the_total_is_unsatisfiable() {
    assert!(matches!(
        parse_range("bytes=100-110", 100),
        Err(Unsatisfiable)
    ));
    assert!(matches!(parse_range("bytes=101-", 100), Err(Unsatisfiable)));
}

#[test]
fn an_empty_total_is_unsatisfiable_for_any_well_formed_range() {
    assert!(matches!(parse_range("bytes=0-5", 0), Err(Unsatisfiable)));
    assert!(matches!(parse_range("bytes=0-", 0), Err(Unsatisfiable)));
    assert!(matches!(parse_range("bytes=-5", 0), Err(Unsatisfiable)));
}

#[test]
fn a_suffix_range_takes_the_last_n_bytes() {
    assert!(is_range(&parse_range("bytes=-10", 100).unwrap(), 90, 99));
    // A suffix longer than the body takes the whole body.
    assert!(is_range(&parse_range("bytes=-500", 100).unwrap(), 0, 99));
}

#[test]
fn a_zero_suffix_is_unsatisfiable() {
    assert!(matches!(parse_range("bytes=-0", 100), Err(Unsatisfiable)));
}

#[test]
fn whitespace_around_the_header_and_bounds_is_ignored() {
    assert!(is_range(
        &parse_range("  bytes= 10 - 19 ", 100).unwrap(),
        10,
        19
    ));
}
