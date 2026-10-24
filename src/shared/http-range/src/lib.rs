//! RFC 7233 single-range parsing for audio streaming (spec FR-009..013).

/// One satisfiable byte range of a blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end_inclusive: u64,
}

/// A well-formed but unsatisfiable Range header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsatisfiable;

/// RFC 7233 single-range parsing.
///
/// - `Ok(None)` — absent or malformed header, or multi-range: serve the full
///   body with `200` (spec FR-012).
/// - `Ok(Some(range))` — serve `206` with `Content-Range`.
/// - `Err(())` — well-formed but unsatisfiable: `416` with `bytes */total`
///   (spec FR-011).
pub fn parse_range(header: &str, total: u64) -> Result<Option<ByteRange>, Unsatisfiable> {
    let header = header.trim();
    let Some(spec) = header.strip_prefix("bytes=") else {
        return Ok(None);
    };
    if spec.contains(',') {
        return Ok(None);
    }
    // A malformed range spec (no `-`, or a non-numeric bound) is not a
    // well-formed-but-unsatisfiable range per FR-012: ignore it and serve
    // the full body, matching absent/multi-range handling above.
    let Some((start_text, end_text)) = spec.split_once('-') else {
        return Ok(None);
    };
    match (start_text.trim(), end_text.trim()) {
        ("", "") => Ok(None),
        // Suffix form: the last N bytes.
        ("", suffix_text) => {
            let Ok(suffix) = suffix_text.parse::<u64>() else {
                return Ok(None);
            };
            if suffix == 0 || total == 0 {
                return Err(Unsatisfiable);
            }
            let start = total - suffix.min(total);
            Ok(Some(ByteRange {
                start,
                end_inclusive: total - 1,
            }))
        }
        (start_text, "") => {
            let Ok(start) = start_text.parse::<u64>() else {
                return Ok(None);
            };
            if total == 0 || start >= total {
                return Err(Unsatisfiable);
            }
            Ok(Some(ByteRange {
                start,
                end_inclusive: total - 1,
            }))
        }
        (start_text, end_text) => {
            let (Ok(start), Ok(end)) = (start_text.parse::<u64>(), end_text.parse::<u64>()) else {
                return Ok(None);
            };
            if total == 0 || start >= total || start > end {
                return Err(Unsatisfiable);
            }
            Ok(Some(ByteRange {
                start,
                end_inclusive: end.min(total - 1),
            }))
        }
    }
}

#[cfg(test)]
#[path = "lib.tests.rs"]
mod tests;
