//! Small text helpers for messages.

/// SPEC Appendix A `key`: a lowercase letter, then lowercase letters, digits,
/// or hyphens.
pub(super) fn is_key(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Joins values as `a, b, c`.
pub(super) fn list<S: AsRef<str>>(values: &[S]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Joins values in backticks: `` `a`, `b` ``.
pub(super) fn quoted_list<S: AsRef<str>>(values: &[S]) -> String {
    values
        .iter()
        .map(|v| format!("`{}`", v.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Joins values in backticks with "and": `` `a` and `b` ``, `` `a`, `b`, and `c` ``.
pub(super) fn quoted_and<S: AsRef<str>>(values: &[S]) -> String {
    let quoted: Vec<String> = values.iter().map(|v| format!("`{}`", v.as_ref())).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [init @ .., last] => format!("{}, and {last}", init.join(", ")),
    }
}

/// Levenshtein distance, for did-you-mean suggestions.
fn edit_distance(a: &str, b: &str) -> usize {
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

/// The closest candidate to `key`, if it's close enough to be a typo.
pub(super) fn suggest<'a>(
    key: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let len = key.chars().count();
    let limit = if len <= 3 { 1 } else { (len / 3).clamp(2, 3) };
    candidates
        .into_iter()
        .map(|c| (edit_distance(key, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins() {
        assert_eq!(quoted_and(&["a"]), "`a`");
        assert_eq!(quoted_and(&["a", "b"]), "`a` and `b`");
        assert_eq!(quoted_and(&["a", "b", "c"]), "`a`, `b`, and `c`");
        assert_eq!(quoted_list(&["a", "b"]), "`a`, `b`");
        assert_eq!(list(&["a", "b"]), "a, b");
    }

    #[test]
    fn suggestions() {
        assert_eq!(suggest("typ", ["type"]), Some("type"));
        assert_eq!(suggest("colour", ["type"]), None);
    }
}
