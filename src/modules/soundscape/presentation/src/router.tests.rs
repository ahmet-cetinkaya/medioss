use crate::router::parse_range;

#[test]
fn parses_single_byte_and_suffix_ranges() {
    assert_eq!(parse_range("bytes=0-0", 10), Some(Ok((0, 0))));
    assert_eq!(parse_range("bytes=-3", 10), Some(Ok((7, 9))));
    assert_eq!(parse_range("bytes=4-", 10), Some(Ok((4, 9))));
}

#[test]
fn ignores_malformed_and_rejects_unsatisfiable_ranges() {
    assert_eq!(parse_range("items=0-1", 10), None);
    assert_eq!(parse_range("bytes=bogus", 10), None);
    assert_eq!(parse_range("bytes=99-100", 10), Some(Err(())));
}
