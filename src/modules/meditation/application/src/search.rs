use unicode_normalization::UnicodeNormalization;

pub fn fold(text: &str) -> String {
    text.nfkd()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn matches(query: &str, haystack: &str) -> bool {
    let folded_query = fold(query);
    let folded_haystack = fold(haystack);
    folded_query
        .split_whitespace()
        .all(|term| folded_haystack.contains(term))
}

#[cfg(test)]
#[path = "search.tests.rs"]
mod tests;
