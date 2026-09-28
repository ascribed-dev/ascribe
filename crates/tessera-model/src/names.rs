//! Name grammars (content-model.md §1.2) and small text helpers.

/// SPEC Appendix A `name-word`: a letter, then letters, digits, `_`, or `-`.
pub fn is_name_word(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// SPEC Appendix A `key`: a lowercase letter, then lowercase letters, digits, or `-`.
pub fn is_key(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// SPEC Appendix A `widget-name`: lowercase words of letters and digits
/// joined by single hyphens, with at least one hyphen, starting with a letter.
pub fn is_widget_name(s: &str) -> bool {
    let words: Vec<&str> = s.split('-').collect();
    words.len() >= 2
        && words.iter().all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
        && s.starts_with(|c: char| c.is_ascii_lowercase())
}

/// Build names: a letter, then letters, digits, `_`, `-`, or `.`.
pub fn is_build_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// SPEC Appendix A `version`: `1*DIGIT *( "." 1*DIGIT )`.
pub fn is_version(s: &str) -> bool {
    !s.is_empty()
        && s.split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// The rule text for `model-invalid-name`'s `{rule}`.
pub const KEY_RULE: &str = "use a lowercase letter, then lowercase letters, digits, or hyphens";
/// The rule text for name-words.
pub const NAME_WORD_RULE: &str = "use a letter, then letters, digits, underscores, or hyphens";
/// The rule text for widget names.
pub const WIDGET_RULE: &str =
    "use lowercase words of letters and digits joined by hyphens, with at least one hyphen";
/// The rule text for build names.
pub const BUILD_RULE: &str = "use a letter, then letters, digits, underscores, hyphens, or dots";

/// Levenshtein distance, for did-you-mean suggestions.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The closest candidate to `key`, if it is close enough to be a typo.
pub fn suggest<'a>(key: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let len = key.chars().count();
    // A transposition costs 2, so short-but-not-tiny keys allow 2.
    let limit = if len <= 3 { 1 } else { (len / 3).clamp(2, 3) };
    candidates
        .into_iter()
        .map(|c| (edit_distance(key, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

/// Joins values for a message, like `a, b, c`.
pub fn list<S: AsRef<str>>(values: &[S]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammars() {
        assert!(is_name_word("self-managed") && is_name_word("A_1"));
        assert!(!is_name_word("1a") && !is_name_word("") && !is_name_word("a b"));
        assert!(is_key("pm") && !is_key("Pm") && !is_key("a_b"));
        assert!(is_widget_name("quill-labspace") && !is_widget_name("labspace"));
        assert!(!is_widget_name("a--b") && !is_widget_name("-a-b") && !is_widget_name("A-b"));
        assert!(is_build_name("self-managed-3.3") && !is_build_name("3.3"));
        assert!(is_version("3.4.1") && !is_version("3.") && !is_version("a"));
    }

    #[test]
    fn suggestions() {
        assert_eq!(
            suggest("content-rot", ["content-root", "output-dir"]),
            Some("content-root")
        );
        assert_eq!(suggest("zzzz", ["content-root"]), None);
    }
}
