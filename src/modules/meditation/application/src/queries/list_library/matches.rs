use super::fold::fold;

pub(crate) fn matches(query: &str, haystack: &str) -> bool {
    let folded_query = fold(query);
    let folded_haystack = fold(haystack);
    folded_query
        .split_whitespace()
        .all(|term| folded_haystack.contains(term))
}

#[cfg(test)]
#[path = "matches.tests.rs"]
mod tests;
