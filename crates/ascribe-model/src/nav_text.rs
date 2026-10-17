//! Editing `ascribe.nav`'s text in place, keeping its comments and layout:
//! the pieces the checks' fixes and the editor's edits are made of. Each
//! works on the text and what [`NavFile::parse`] read from it.
//!
//! [`NavFile::parse`]: crate::NavFile::parse

use ascribe_core::{Span, TextEdit};

/// A TOML basic string of `text`.
pub fn toml_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The start of the line `at` is on.
pub fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// The end of the line `at` is on, after its line break.
pub fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i + 1)
}

/// Whether a line, without its break, is a table's header.
fn is_header(line: &str) -> bool {
    line.trim_start().starts_with('[')
}

/// The text a `[[sections]]` table whose `folder` is at `folder` takes up:
/// from its header's line to the next header, less the comments and blank
/// lines right before that one, which are about it. `None` for a table
/// written inline.
pub fn table_span(text: &str, folder: Span) -> Option<Span> {
    let mut start = None;
    let mut at = line_start(text, folder.start());
    loop {
        let line = &text[at..line_end(text, at)];
        if is_header(line) {
            let compact: String = line.split_whitespace().collect();
            if compact == "[[sections]]" || compact.starts_with("[[sections]]#") {
                start = Some(at);
            }
            break;
        }
        if at == 0 {
            break;
        }
        at = line_start(text, at - 1);
    }
    let start = start?;
    // The next header after the table's own.
    let mut end = text.len();
    let mut at = line_end(text, start);
    while at < text.len() {
        let next = line_end(text, at);
        if is_header(&text[at..next]) {
            end = at;
            break;
        }
        at = next;
    }
    // The comments right above the next header are its own.
    if end < text.len() {
        while end > start {
            let previous = line_start(text, end - 1);
            let line = text[previous..end].trim();
            if line.starts_with('#') {
                end = previous;
            } else {
                break;
            }
        }
    }
    Some(Span::new(start, end))
}

/// [`table_span`] with the comments right above the table's header, which
/// are about it: what moves when the table moves.
pub fn table_span_with_comments(text: &str, folder: Span) -> Option<Span> {
    let span = table_span(text, folder)?;
    let mut start = span.start();
    while start > 0 {
        let previous = line_start(text, start - 1);
        if text[previous..start].trim_start().starts_with('#') {
            start = previous;
        } else {
            break;
        }
    }
    Some(Span::new(start, span.end()))
}

/// The text to take out with the `i`th of a `pages` list: its own lines
/// when it starts one, else from the end of the entry before it.
pub fn item_span(text: &str, pages: &[(String, Span)], i: usize) -> Option<Span> {
    let span = pages.get(i)?.1;
    let start = line_start(text, span.start());
    if text[start..span.start()].trim().is_empty() {
        // Its line, with its comma and any comment after it.
        let end = line_end(text, span.end());
        let rest = text[span.end()..end].trim();
        if rest.is_empty() || rest.starts_with(',') || rest.starts_with('#') {
            return Some(Span::new(start, end));
        }
    }
    let previous = pages.get(i.checked_sub(1)?)?.1;
    Some(Span::new(previous.end(), span.end()))
}

/// The text to take out with the `i`th of a `pages` list, whichever it is:
/// as [`item_span`] has it, and the first of a list written on one line up
/// to the next entry.
pub fn remove_item_span(text: &str, pages: &[(String, Span)], i: usize) -> Option<Span> {
    if let Some(span) = item_span(text, pages, i) {
        return Some(span);
    }
    let span = pages.get(i)?.1;
    match pages.get(i + 1) {
        Some((_, next)) => Some(Span::new(span.start(), next.start())),
        None => Some(span),
    }
}

/// The edit that puts `name` before the `i`th entry of a `pages` list, in
/// the list's own layout: on the entry's line when it's written on one,
/// else on a line of its own.
pub fn insert_before_item(
    text: &str,
    pages: &[(String, Span)],
    i: usize,
    name: &str,
) -> Option<TextEdit> {
    let span = pages.get(i)?.1;
    let start = line_start(text, span.start());
    let quoted = toml_string(name);
    if text[start..span.start()].trim().is_empty() {
        let indent = &text[start..span.start()];
        return Some(TextEdit::insert(start, format!("{indent}{quoted},\n")));
    }
    Some(TextEdit::insert(span.start(), format!("{quoted}, ")))
}

/// The edit that appends `names` to a `pages` list at `list`, whose
/// entries are `written`, in the list's own layout: one a line when it's
/// written so, else on its line.
pub fn append_names(
    text: &str,
    list: Span,
    written: &[(String, Span)],
    names: &[&str],
) -> Option<TextEdit> {
    if names.is_empty() {
        return None;
    }
    let names: Vec<String> = names.iter().map(|n| toml_string(n)).collect();
    let close = list.end().checked_sub(1)?;
    let Some((_, last)) = written.last() else {
        // An empty list: the names inside its brackets.
        return Some(TextEdit::insert(close, names.join(", ")));
    };
    let after = &text[last.end()..close];
    if !after.contains('\n') {
        let added: String = names.iter().map(|n| format!(", {n}")).collect();
        return Some(TextEdit::insert(last.end(), added));
    }
    let line = line_start(text, last.start());
    let indent = &text[line..last.start()];
    if after.trim_start().starts_with(',') {
        // A comma after each, the last included: after the last's line.
        let at = line_end(text, last.end());
        let added: String = names.iter().map(|n| format!("{indent}{n},\n")).collect();
        return Some(TextEdit::insert(at, added));
    }
    let added: String = names.iter().map(|n| format!(",\n{indent}{n}")).collect();
    Some(TextEdit::insert(last.end(), added))
}

/// The edit that adds `line` (a `key = value`, without its break) to the
/// table whose `folder` is at `folder`: after the `folder` line. `None` for
/// a table written inline.
pub fn add_to_table(text: &str, folder: Span, line: &str) -> Option<TextEdit> {
    table_span(text, folder)?;
    let at = line_end(text, folder.end());
    let lead = if at == text.len() && !text.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    Some(TextEdit::insert(at, format!("{lead}{line}\n")))
}

/// The edit that adds `tables`, each a `[[sections]]` table's text ending in
/// a line break, at the end of the file, a blank line before each.
pub fn append_tables(text: &str, tables: &[String]) -> TextEdit {
    let mut added = String::new();
    if !text.is_empty() && !text.ends_with('\n') {
        added.push('\n');
    }
    for table in tables {
        if !(added.is_empty() && text.is_empty()) {
            added.push('\n');
        }
        added.push_str(table);
    }
    TextEdit::insert(text.len(), added)
}

/// A `[[sections]]` table for `folder`, with `pages` and `label` when given.
pub fn section_table(folder: &str, pages: Option<&[&str]>, label: Option<&str>) -> String {
    let mut table = format!("[[sections]]\nfolder = {}\n", toml_string(folder));
    if let Some(pages) = pages {
        table.push_str(&pages_line(pages));
        table.push('\n');
    }
    if let Some(label) = label {
        table.push_str(&format!("label = {}\n", toml_string(label)));
    }
    table
}

/// `pages = [...]` with `names`, on one line.
pub fn pages_line(names: &[&str]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| toml_string(n)).collect();
    format!("pages = [{}]", quoted.join(", "))
}

/// The span of the line of a table's `key`, with its break, when the
/// table whose `folder` is at `folder` writes it on a line of its own.
pub fn key_line(text: &str, folder: Span, key: &str) -> Option<Span> {
    let table = table_span(text, folder)?;
    let mut at = line_end(text, table.start());
    while at < table.end() {
        let end = line_end(text, at);
        let line = text[at..end].trim_start();
        if let Some(rest) = line.strip_prefix(key)
            && rest.trim_start().starts_with('=')
        {
            return Some(Span::new(at, end));
        }
        at = end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NavFile;
    use ascribe_core::{FileId, apply_edits};

    /// `text` with `left` appended to its only section's `pages`.
    fn appended(text: &str, left: &[&str]) -> String {
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        let entry = &nav.sections[0];
        let edit = append_names(
            text,
            entry.pages_span.expect("a list"),
            entry.pages.as_deref().expect("pages"),
            left,
        )
        .expect("an edit");
        apply_edits(text, &[edit]).expect("it applies")
    }

    #[test]
    fn names_are_appended_in_the_lists_layout() {
        let head = "version = 1\n[[sections]]\nfolder = \"\"\n";
        assert_eq!(
            appended(&format!("{head}pages = [\"a.md\"]\n"), &["b.md", "c d.md"]),
            format!("{head}pages = [\"a.md\", \"b.md\", \"c d.md\"]\n")
        );
        assert_eq!(
            appended(&format!("{head}pages = []\n"), &["b.md", "c.md"]),
            format!("{head}pages = [\"b.md\", \"c.md\"]\n")
        );
        assert_eq!(
            appended(
                &format!("{head}pages = [\n  \"a.md\", # first\n]\n"),
                &["b.md"]
            ),
            format!("{head}pages = [\n  \"a.md\", # first\n  \"b.md\",\n]\n")
        );
        assert_eq!(
            appended(&format!("{head}pages = [\n  \"a.md\"\n]\n"), &["b.md"]),
            format!("{head}pages = [\n  \"a.md\",\n  \"b.md\"\n]\n")
        );
    }

    fn pages(text: &str) -> Vec<(String, Span)> {
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        nav.sections[0].pages.clone().expect("pages")
    }

    #[test]
    fn a_second_entry_is_taken_out_with_its_own_text() {
        let take = |text: &str, i: usize| {
            let span = item_span(text, &pages(text), i).expect("a span");
            apply_edits(text, &[TextEdit::delete(span)]).expect("it applies")
        };
        let head = "version = 1\n[[sections]]\nfolder = \"\"\n";
        assert_eq!(
            take(
                &format!("{head}pages = [\"a.md\", \"b.md\", \"a.md\"]\n"),
                2
            ),
            format!("{head}pages = [\"a.md\", \"b.md\"]\n")
        );
        assert_eq!(
            take(
                &format!("{head}pages = [\"a.md\", \"a.md\", \"b.md\"]\n"),
                1
            ),
            format!("{head}pages = [\"a.md\", \"b.md\"]\n")
        );
        assert_eq!(
            take(
                &format!("{head}pages = [\n  \"a.md\",\n  \"a.md\", # again\n  \"b.md\",\n]\n"),
                1
            ),
            format!("{head}pages = [\n  \"a.md\",\n  \"b.md\",\n]\n")
        );
    }

    #[test]
    fn any_entry_is_taken_out_and_put_back_in_the_lists_layout() {
        let head = "version = 1\n[[sections]]\nfolder = \"\"\n";
        let one_line = format!("{head}pages = [\"a.md\", \"b.md\", \"c.md\"]\n");
        let take = |text: &str, i: usize| {
            let span = remove_item_span(text, &pages(text), i).expect("a span");
            apply_edits(text, &[TextEdit::delete(span)]).expect("it applies")
        };
        assert_eq!(
            take(&one_line, 0),
            format!("{head}pages = [\"b.md\", \"c.md\"]\n")
        );
        let only = format!("{head}pages = [\"a.md\"]\n");
        assert_eq!(take(&only, 0), format!("{head}pages = []\n"));
        let put = |text: &str, i: usize| {
            let edit = insert_before_item(text, &pages(text), i, "new.md").expect("an edit");
            apply_edits(text, &[edit]).expect("it applies")
        };
        assert_eq!(
            put(&one_line, 1),
            format!("{head}pages = [\"a.md\", \"new.md\", \"b.md\", \"c.md\"]\n")
        );
        let lines = format!("{head}pages = [\n  \"a.md\",\n  \"b.md\", # second\n]\n");
        assert_eq!(
            put(&lines, 1),
            format!("{head}pages = [\n  \"a.md\",\n  \"new.md\",\n  \"b.md\", # second\n]\n")
        );
    }

    #[test]
    fn a_second_table_is_taken_out_leaving_the_next_ones_comments() {
        let text = "version = 1\n\n[[sections]]\nfolder = \"a/\"\n\n[[sections]]\nfolder = \"a/\"\nlabel = \"A\"\n\n# The guides.\n[[sections]]\nfolder = \"b/\"\n";
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        let span = table_span(text, nav.sections[1].folder_span).expect("a table");
        assert_eq!(
            apply_edits(text, &[TextEdit::delete(span)]).expect("it applies"),
            "version = 1\n\n[[sections]]\nfolder = \"a/\"\n\n# The guides.\n[[sections]]\nfolder = \"b/\"\n"
        );
        let with = table_span_with_comments(text, nav.sections[2].folder_span).expect("a table");
        assert_eq!(
            &text[with.range()],
            "# The guides.\n[[sections]]\nfolder = \"b/\"\n"
        );
        let last = "version = 1\n[[sections]]\nfolder = \"a/\"\n[[sections]]\nfolder = \"a/\"\n";
        let nav = NavFile::parse(last, FileId::new(0)).expect("it reads");
        let span = table_span(last, nav.sections[1].folder_span).expect("a table");
        assert_eq!(
            apply_edits(last, &[TextEdit::delete(span)]).expect("it applies"),
            "version = 1\n[[sections]]\nfolder = \"a/\"\n"
        );
    }

    #[test]
    fn keys_and_tables_are_added() {
        let text = "version = 1\n[[sections]]\nfolder = \"a/\" # the first\nlabel = \"A\"\n";
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        let folder = nav.sections[0].folder_span;
        let edit = add_to_table(text, folder, &pages_line(&["b.md"])).expect("an edit");
        assert_eq!(
            apply_edits(text, &[edit]).expect("it applies"),
            "version = 1\n[[sections]]\nfolder = \"a/\" # the first\npages = [\"b.md\"]\nlabel = \"A\"\n"
        );
        let label = key_line(text, folder, "label").expect("a line");
        assert_eq!(&text[label.range()], "label = \"A\"\n");
        let edit = append_tables(text, &[section_table("b/", None, Some("B"))]);
        assert_eq!(
            apply_edits(text, &[edit]).expect("it applies"),
            format!("{text}\n[[sections]]\nfolder = \"b/\"\nlabel = \"B\"\n")
        );
        let edit = append_tables("", &[section_table("", None, None)]);
        assert_eq!(edit.new_text, "[[sections]]\nfolder = \"\"\n");
    }
}
