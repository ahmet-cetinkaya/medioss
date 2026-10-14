pub(crate) fn parse_range(value: &str, total: u64) -> Option<Result<(u64, u64), ()>> {
    let spec = value.trim().strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    match (start.trim(), end.trim()) {
        ("", suffix) => {
            let suffix = suffix.parse::<u64>().ok()?;
            if suffix == 0 || total == 0 {
                Some(Err(()))
            } else {
                Some(Ok((total - suffix.min(total), total - 1)))
            }
        }
        (start, "") => {
            let start = start.parse::<u64>().ok()?;
            if start >= total {
                Some(Err(()))
            } else {
                Some(Ok((start, total - 1)))
            }
        }
        (start, end) => {
            let start = start.parse::<u64>().ok()?;
            let end = end.parse::<u64>().ok()?;
            if total == 0 || start >= total || start > end {
                Some(Err(()))
            } else {
                Some(Ok((start, end.min(total - 1))))
            }
        }
    }
}
