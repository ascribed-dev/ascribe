//! Lines shaped like directives with unknown names (SPEC §3.2).
//!
//! `@warning:` isn't a directive, so the line is text; but it has the shape
//! of one, so processors SHOULD warn and suggest the closest known name. The
//! line stays text in the tree; this module only finds the shape and picks
//! the suggestion.

use crate::options::ParseOptions;
use ascribe_core::END_KEYWORD;

/// If `line` (starting at its `@`, after any container prefix) has the shape
/// of a directive line, the length of its name (without the `@`).
///
/// The shape is `@` and a name (lowercase letters, digits, and hyphens,
/// starting with a letter) followed, after any spaces and tabs, by `{`, `:`,
/// or the end of the line.
pub(crate) fn directive_shape(line: &str) -> Option<usize> {
    let rest = line.strip_prefix('@')?;
    let bytes = rest.as_bytes();
    if !bytes.first().is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let len = bytes
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        .count();
    // Spaces and tabs may come before the `{` or `:`, as they may in a
    // directive line (SPEC §3.1).
    match rest[len..].trim_start_matches([' ', '\t']).bytes().next() {
        None | Some(b'{' | b':') => Some(len),
        Some(_) => None,
    }
}

/// Whether `name` is a keyword: a schema's name, or `end`.
pub(crate) fn is_known(name: &str, options: &ParseOptions) -> bool {
    name == END_KEYWORD || options.schema(name).is_some()
}

/// The suggestion for a directive-shaped line with an unknown `name`, as the
/// text to write instead: `@note {type=warning}:` when the name is a note
/// type (SPEC §3.2's example), otherwise the closest known keyword.
pub(crate) fn suggest(name: &str, options: &ParseOptions) -> Option<String> {
    if options.schema("note").is_some() && options.note_types.iter().any(|t| t == name) {
        return Some(format!("@note {{type={name}}}:"));
    }
    let limit = if name.len() >= 6 { 2 } else { 1 };
    options
        .schemas
        .iter()
        .map(|s| s.name.as_str())
        .chain(std::iter::once(END_KEYWORD))
        .map(|candidate| (edit_distance(name, candidate), candidate))
        .filter(|&(distance, _)| distance <= limit)
        .min_by_key(|&(distance, candidate)| (distance, candidate))
        .map(|(_, candidate)| format!("@{candidate}"))
}

/// The Levenshtein distance between two strings, by characters.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(ca != cb))
                .min(row[j] + 1)
                .min(above + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_shape() {
        assert_eq!(directive_shape("@warning:"), Some(7));
        assert_eq!(directive_shape("@warning: text"), Some(7));
        assert_eq!(directive_shape("@warning {type=x}"), Some(7));
        assert_eq!(directive_shape("@warning"), Some(7));
        assert_eq!(directive_shape("@warning  \t"), Some(7));
        assert_eq!(directive_shape("@quill-x:"), Some(7));
    }

    #[test]
    fn prose_isnt_the_shape() {
        for line in [
            "@astrojs/react",
            "@warning some text",
            "@Warning:",
            "@",
            "@:",
            "@1abc:",
            "\\@warning:",
            "support@example.com",
            "@timestamp is",
        ] {
            assert_eq!(directive_shape(line), None, "{line:?}");
        }
    }

    #[test]
    fn suggests_a_note_type_or_the_closest_name() {
        let options = ParseOptions::default();
        assert_eq!(
            suggest("warning", &options).as_deref(),
            Some("@note {type=warning}:")
        );
        assert_eq!(suggest("notes", &options).as_deref(), Some("@note"));
        assert_eq!(
            suggest("availible", &options).as_deref(),
            Some("@available")
        );
        assert_eq!(suggest("inlcude", &options).as_deref(), Some("@include"));
        assert_eq!(suggest("astrojs", &options), None);
        assert_eq!(suggest("timestamp", &options), None);
    }

    #[test]
    fn measures_edit_distance() {
        assert_eq!(edit_distance("note", "note"), 0);
        assert_eq!(edit_distance("note", "nots"), 1);
        assert_eq!(edit_distance("note", ""), 4);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }
}
