use unicode_normalization::UnicodeNormalization;

pub(crate) fn fold(text: &str) -> String {
    text.nfkd()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
#[path = "fold.tests.rs"]
mod tests;
