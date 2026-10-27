//! A declared phrase's value written out in a page's text, where the phrase
//! could be (`phrase-text-written`).

use ascribe_core::{Applicability, Fix, Issue, Span, TextEdit, diagnostics};
use ascribe_model::ContentModel;

use super::Ctx;

/// What the text being walked is, for this check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Written {
    /// Text whose value can be a phrase, outside a heading or in one with an
    /// `@id`.
    Prose,
    /// A heading with no `@id`, whose link would change with the value once
    /// it's a phrase.
    Heading,
    /// Text that isn't looked in: a heading marked `@term`, which names the
    /// term itself, and the label of a reference link, which a phrase would
    /// change.
    Not,
}

/// The values the check looks for, longest first, so that a value inside
/// another (`Quill` in `Quill Cloud`) loses to it. Of two phrases with one
/// value, the one declared first is named.
pub(super) fn values(model: &ContentModel) -> Vec<(String, String)> {
    if model.checks.is_off(diagnostics::PHRASE_TEXT_WRITTEN) {
        return Vec::new();
    }
    let mut values: Vec<(String, String)> = Vec::new();
    for phrase in &model.phrases {
        // One character, or a number alone (`24`), is too often another
        // word or number to mean the phrase.
        let value = phrase.value.as_str();
        if value.chars().count() < 2
            || value.trim() != value
            || value.chars().all(|c| c.is_ascii_digit())
        {
            continue;
        }
        if !values.iter().any(|(v, _)| *v == phrase.value) {
            values.push((phrase.value.clone(), phrase.key.clone()));
        }
    }
    // A stable sort keeps declaration order among values of one length.
    values.sort_by_key(|(v, _)| std::cmp::Reverse(v.len()));
    values
}

/// Where `text` writes out one of `values`, as `(start, end, index)`, in
/// order: whole words only, and of a value inside another, the longer.
fn written(text: &str, values: &[(String, String)]) -> Vec<(usize, usize, usize)> {
    let mut found: Vec<(usize, usize, usize)> = Vec::new();
    for (index, (value, _)) in values.iter().enumerate() {
        for (at, _) in text.match_indices(value.as_str()) {
            let end = at + value.len();
            let overlaps = found.iter().any(|&(s, e, _)| at < e && s < end);
            if !overlaps && whole(text, at, end) {
                found.push((at, end, index));
            }
        }
    }
    found.sort_unstable();
    found
}

/// `text` with each phrase's value that `phrase-text-written` would report
/// written as the phrase, for text Ascribe writes into a page, such as an
/// editor's placeholder. Unchanged when the check is off.
pub fn with_phrases(text: &str, model: &ContentModel) -> String {
    let values = values(model);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (at, end, index) in written(text, &values) {
        out.push_str(&text[last..at]);
        out.push('{');
        out.push_str(&values[index].1);
        out.push('}');
        last = end;
    }
    out.push_str(&text[last..]);
    out
}

impl Ctx<'_> {
    /// The values written out in one text node, its source at `span`.
    pub(super) fn check_phrase_text(&mut self, span: Span) {
        if self.written == Written::Not || self.phrase_values.is_empty() {
            return;
        }
        let found = written(&self.file.text[span.range()], &self.phrase_values);
        let heading = self.written == Written::Heading;
        for (at, end, index) in found {
            let (value, key) = &self.phrase_values[index];
            let phrase = format!("{{{key}}}");
            let at = Span::new(span.start() + at, span.start() + end);
            // A value that can't be an ordinary word means the phrase where
            // it's written; one that can may mean the word. A heading with
            // no `@id` would have a phrase without one.
            let applicability = if heading || ordinary_word(value) {
                Applicability::Unsafe
            } else {
                Applicability::Safe
            };
            let mut issue = Issue::new(diagnostics::PHRASE_TEXT_WRITTEN, self.location(at))
                .with_arg("text", value.clone())
                .with_arg("key", phrase.clone())
                .with_fix(Fix {
                    title: format!("Replace with `{phrase}`"),
                    file: self.id,
                    edits: vec![TextEdit::replace(at, phrase)],
                    applicability,
                });
            if heading {
                issue = issue.with_variant("heading");
            }
            self.report(issue);
        }
    }
}

/// Whether a value could be an ordinary word: one word of letters and
/// digits, which may have a hyphen or an apostrophe in it.
fn ordinary_word(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '\'' || c == '’')
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `source[at..end]` stands on its own: not part of a longer word or
/// version (`Quills`, `3.4.10`, `3.4.1-beta`), and not where the phrase would
/// read otherwise, after a backslash, which would escape it, or between
/// braces, which would double them. A version may follow a `v`, as in
/// `v3.4.1`.
fn whole(source: &str, at: usize, end: usize) -> bool {
    let value = &source[at..end];
    let mut before = source[..at].chars().rev();
    let mut after = source[end..].chars();
    let (first, last) = match (value.chars().next(), value.chars().next_back()) {
        (Some(first), Some(last)) => (first, last),
        _ => return false,
    };
    let previous = before.next();
    if matches!(previous, Some('\\' | '{')) {
        return false;
    }
    if let Some(p) = previous {
        if is_word(first) && is_word(p) {
            let versioned = first.is_ascii_digit()
                && matches!(p, 'v' | 'V')
                && !before.next().is_some_and(is_word);
            if !versioned {
                return false;
            }
        } else if matches!(p, '.' | '-') && is_word(first) && before.next().is_some_and(is_word) {
            return false;
        }
    }
    match after.next() {
        None => true,
        Some('}') => false,
        Some(n) if is_word(last) && is_word(n) => false,
        Some('.' | '-') if is_word(last) => !after.next().is_some_and(is_word),
        Some(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(source: &str, value: &str) -> bool {
        let start = source.find(value).unwrap_or_default();
        whole(source, start, start + value.len())
    }

    #[test]
    fn whole_words_only() {
        assert!(at("Use Quill today.", "Quill"));
        assert!(at("Quill", "Quill"));
        assert!(!at("Quills are sharp.", "Quill"));
        assert!(!at("QuillCloud", "Quill"));
        assert!(!at("\\Quill", "Quill"));
        assert!(!at("{Quill}", "Quill"));
    }

    #[test]
    fn versions_stand_alone() {
        assert!(at("Quill 3.4.1 is out.", "3.4.1"));
        assert!(at("Get v3.4.1 now", "3.4.1"));
        assert!(!at("Get 3.4.10 now", "3.4.1"));
        assert!(!at("Get 3.4.1-beta now", "3.4.1"));
        assert!(!at("Get 13.4.1 now", "3.4.1"));
        assert!(!at("Get 2.3.4.1 now", "3.4.1"));
        assert!(!at("Get dev3.4.1 now", "3.4.1"));
    }

    #[test]
    fn a_url_may_go_on() {
        assert!(at(
            "Call https://api.quill.dev/v3/users.",
            "https://api.quill.dev/v3/"
        ));
    }

    #[test]
    fn ordinary_words() {
        assert!(ordinary_word("Quill"));
        assert!(ordinary_word("re-sync"));
        assert!(ordinary_word("v2"));
        assert!(!ordinary_word("Quill Cloud"));
        assert!(!ordinary_word("3.4.1"));
        assert!(!ordinary_word("C++"));
    }
}
