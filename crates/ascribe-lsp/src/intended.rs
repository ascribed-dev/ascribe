//! The quick fix that marks a review check's problem as intended (SPEC
//! §4.9): an acknowledgement written where the check's place says, with a
//! reason for the author to replace.

use ascribe_core::intended::{EntryKind, Place};
use ascribe_core::{Span, TextEdit};
use ascribe_syntax::{Block, BlockKind, Bound, ParsedDocument, frontmatter_lines};

/// The quick fix's title.
pub(crate) const TITLE: &str = "Mark as intended, and write why";

/// The reason the acknowledgement is written with, for the author to
/// replace: the fix can't know why, and the checks report it until it's
/// replaced.
const REASON: &str = ascribe_core::intended::PLACEHOLDER_REASON;

/// Where an acknowledgement is written.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    /// In the page's own text.
    Source(TextEdit),
    /// In `ascribe.toml`.
    Model(TextEdit),
}

/// The edit that acknowledges a problem of `check` located at `at` in
/// `source`, written at `place`. `subject` is the entry the problem names
/// (`phrase intro`), for a check acknowledged in the content model. `None`
/// when the source doesn't allow it, such as frontmatter written in flow
/// style.
pub(crate) fn mark(
    place: Place,
    check: &str,
    source: &str,
    document: &ParsedDocument,
    at: usize,
    subject: Option<&str>,
    model_text: &str,
) -> Option<Mark> {
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    match place {
        Place::Page => page(check, source, document, newline).map(Mark::Source),
        Place::Block => block(check, source, document, at, newline).map(Mark::Source),
        Place::Entry => entry(check, subject?, model_text).map(Mark::Model),
    }
}

/// An entry in the page's `intended` frontmatter, which is added, as is the
/// frontmatter, when there's none.
fn page(check: &str, source: &str, document: &ParsedDocument, nl: &str) -> Option<TextEdit> {
    let item = |indent: &str| format!("{indent}- check: {check}{nl}{indent}  reason: {REASON}{nl}");
    let Some(frontmatter) = &document.frontmatter else {
        return Some(TextEdit::insert(
            0,
            format!("---{nl}intended:{nl}{}---{nl}", item("  ")),
        ));
    };
    let content = frontmatter.content;
    let text = source.get(content.start()..content.end())?;
    let lines = frontmatter_lines(text);
    let Some(key) = lines
        .iter()
        .position(|l| l.indent == 0 && l.item.is_none() && l.key == Some("intended"))
    else {
        return Some(TextEdit::insert(
            content.end(),
            format!("intended:{nl}{}", item("  ")),
        ));
    };
    // Only a block sequence takes another item.
    if !lines[key].value.is_empty() {
        return None;
    }
    let items = &lines[key + 1..];
    let end = items
        .iter()
        .position(|l| l.indent == 0 && l.item.is_none())
        .map_or(text.len(), |i| items[i].start);
    let indent = items
        .first()
        .filter(|l| l.start < end)
        .and_then(|l| l.item)
        .unwrap_or(2);
    Some(TextEdit::insert(
        content.start() + end,
        item(&" ".repeat(indent)),
    ))
}

/// An `@intended` line directly above the outermost block the problem is
/// in, at its indentation; for a heading, at the top of its section, after
/// the directives already there.
fn block(
    check: &str,
    source: &str,
    document: &ParsedDocument,
    at: usize,
    nl: &str,
) -> Option<TextEdit> {
    let i = document
        .blocks
        .iter()
        .position(|b| b.span.start() <= at && at <= b.span.end())?;
    let block = &document.blocks[i];
    if matches!(block.kind, BlockKind::Heading(_)) {
        return under_heading(check, source, &document.blocks, i, nl);
    }
    let start = source
        .get(..block.span.start())?
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let indent = source.get(start..block.span.start())?;
    if !indent.chars().all(|c| c == ' ' || c == '\t') {
        return None;
    }
    Some(TextEdit::insert(
        start,
        format!("{indent}@intended {{check={check}}}: {REASON}{nl}"),
    ))
}

/// An `@intended` line at the top of the section of the heading
/// `blocks[heading]`, after the directives that bind it.
fn under_heading(
    check: &str,
    source: &str,
    blocks: &[Block],
    heading: usize,
    nl: &str,
) -> Option<TextEdit> {
    let last = blocks[heading + 1..]
        .iter()
        .take_while(|b| b.binding() == Some(Bound::Heading))
        .last()
        .unwrap_or(&blocks[heading]);
    let end = last.span.end();
    let start = source
        .get(..blocks[heading].span.start())?
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let indent = source.get(start..blocks[heading].span.start())?;
    if !indent.chars().all(|c| c == ' ' || c == '\t') {
        return None;
    }
    let line = format!("{indent}@intended {{check={check}}}: {REASON}");
    if source.get(..end)?.ends_with('\n') {
        return Some(TextEdit::insert(end, format!("{line}{nl}")));
    }
    Some(match source.get(end..)?.find('\n') {
        Some(i) => TextEdit::insert(end + i + 1, format!("{line}{nl}")),
        None => TextEdit::insert(source.len(), format!("{nl}{line}")),
    })
}

/// An `[[intended]]` table at the end of `ascribe.toml`.
pub(crate) fn entry(check: &str, subject: &str, model_text: &str) -> Option<TextEdit> {
    let (key, name) = subject.split_once(' ')?;
    let kind = EntryKind::from_key(key)?;
    let nl = if model_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let lead = if model_text.is_empty() || model_text.ends_with('\n') {
        ""
    } else {
        nl
    };
    let quoted = name.replace('\\', "\\\\").replace('"', "\\\"");
    Some(TextEdit {
        span: Span::empty(model_text.len()),
        new_text: format!(
            "{lead}{nl}[[intended]]{nl}check = \"{check}\"{nl}{} = \"{quoted}\"{nl}reason = \"{REASON}\"{nl}",
            kind.key()
        ),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use ascribe_core::schema::builtin_schemas;
    use ascribe_syntax::ParseOptions;

    use super::*;

    fn apply(text: &str, place: Place, at: &str, subject: Option<&str>) -> String {
        let document = ascribe_syntax::parse(text, &ParseOptions::new(builtin_schemas()));
        let at = text.find(at).unwrap();
        let (mut out, edit) =
            match mark(place, "some-check", text, &document, at, subject, "").unwrap() {
                Mark::Source(edit) => (text.to_owned(), edit),
                Mark::Model(edit) => (String::new(), edit),
            };
        out.replace_range(edit.span.start()..edit.span.end(), &edit.new_text);
        out
    }

    #[test]
    fn a_block_gets_a_line_above_it() {
        assert_eq!(
            apply(
                "# Title\n\nOne.\nTwo.\n\nThree.\n",
                Place::Block,
                "Two",
                None
            ),
            "# Title\n\n@intended {check=some-check}: why this is intended\nOne.\nTwo.\n\nThree.\n"
        );
    }

    #[test]
    fn a_heading_gets_a_line_at_the_top_of_its_section() {
        assert_eq!(
            apply(
                "# Title\n\n## Region\n@term\n\nA place.\n",
                Place::Block,
                "Region",
                None
            ),
            "# Title\n\n## Region\n@term\n@intended {check=some-check}: why this is intended\n\nA place.\n"
        );
        assert_eq!(
            apply("## Region", Place::Block, "Region", None),
            "## Region\n@intended {check=some-check}: why this is intended"
        );
    }

    #[test]
    fn a_page_gets_frontmatter_or_an_entry_in_it() {
        assert_eq!(
            apply("# Title\n", Place::Page, "Title", None),
            "---\nintended:\n  - check: some-check\n    reason: why this is intended\n---\n# Title\n"
        );
        assert_eq!(
            apply("---\ntitle: T\n---\n# Title\n", Place::Page, "Title", None),
            "---\ntitle: T\nintended:\n  - check: some-check\n    reason: why this is intended\n---\n# Title\n"
        );
        assert_eq!(
            apply(
                "---\nintended:\n- check: other\n  reason: r\ntitle: T\n---\n# Title\n",
                Place::Page,
                "Title",
                None
            ),
            "---\nintended:\n- check: other\n  reason: r\n- check: some-check\n  reason: why this is intended\ntitle: T\n---\n# Title\n"
        );
    }

    #[test]
    fn flow_style_frontmatter_is_left_alone() {
        let text = "---\nintended: []\n---\n# Title\n";
        let document = ascribe_syntax::parse(text, &ParseOptions::new(builtin_schemas()));
        assert_eq!(mark(Place::Page, "c", text, &document, 0, None, ""), None);
    }

    #[test]
    fn an_entry_gets_a_table_in_the_content_model() {
        let text = "# Title\n";
        let document = ascribe_syntax::parse(text, &ParseOptions::new(builtin_schemas()));
        let Some(Mark::Model(edit)) = mark(
            Place::Entry,
            "c",
            text,
            &document,
            0,
            Some("phrase a\"b"),
            "spec = \"0.1\"",
        ) else {
            panic!("no edit");
        };
        assert_eq!(edit.span, Span::empty(12));
        assert_eq!(
            edit.new_text,
            "\n\n[[intended]]\ncheck = \"c\"\nphrase = \"a\\\"b\"\nreason = \"why this is intended\"\n"
        );
    }
}
