//! What `ascribe fmt --check` shows of a file's edits: each change as the
//! lines it touches, before and after, or as a unified diff with `--diff`.

use std::io::{self, Write};

use ascribe_core::TextEdit;

/// The lines of context a unified diff shows around a change, as `diff -u`
/// and `git diff` do.
const CONTEXT: usize = 3;

/// One change: lines of the file replaced by others.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Change {
    /// The first line it replaces, from 0.
    pub line: usize,
    /// The lines it replaces, without their line endings.
    pub before: Vec<String>,
    /// The lines it writes in their place.
    pub after: Vec<String>,
}

/// The changes `edits` make to `source`: each edit widened to the whole
/// lines it touches, and edits that touch the same line made one change.
/// The edits don't overlap, and are in file order.
pub(super) fn changes(source: &str, edits: &[TextEdit]) -> Vec<Change> {
    let starts = line_starts(source);
    let line_of = |offset: usize| starts.partition_point(|&s| s <= offset) - 1;
    let line_end = |line: usize| starts.get(line + 1).copied().unwrap_or(source.len());
    // Each region: the lines it covers, and the edits in it.
    let mut regions: Vec<(usize, usize, Vec<&TextEdit>)> = Vec::new();
    for edit in edits {
        let first = line_of(edit.span.start());
        // An edit that ends at the start of a line, having removed the line
        // ending before it, doesn't touch that line.
        let last = if edit.span.end() > edit.span.start() && starts.contains(&edit.span.end()) {
            line_of(edit.span.end() - 1)
        } else {
            line_of(edit.span.end())
        };
        match regions.last_mut() {
            Some((_, end, within)) if first <= *end => {
                *end = (*end).max(last);
                within.push(edit);
            }
            _ => regions.push((first, last, vec![edit])),
        }
    }
    regions
        .into_iter()
        .map(|(first, last, within)| {
            let start = starts[first];
            let end = line_end(last);
            let mut after = String::new();
            let mut at = start;
            for edit in within {
                after.push_str(&source[at..edit.span.start()]);
                after.push_str(&edit.new_text);
                at = edit.span.end();
            }
            after.push_str(&source[at..end]);
            Change {
                line: first,
                before: lines(&source[start..end]),
                after: lines(&after),
            }
        })
        .collect()
}

/// Writes each change as `file:line`, then its lines as they are (`-`) and
/// as they'd be (`+`), a blank line after each.
pub(super) fn write_lines(out: &mut dyn Write, file: &str, changes: &[Change]) -> io::Result<()> {
    for change in changes {
        writeln!(out, "{file}:{}", change.line + 1)?;
        for line in &change.before {
            writeln!(out, "{}", marked('-', line))?;
        }
        for line in &change.after {
            writeln!(out, "{}", marked('+', line))?;
        }
        writeln!(out)?;
    }
    Ok(())
}

/// A line after its mark and a space; a blank line as the mark alone, so
/// no line ends in a space.
fn marked(mark: char, line: &str) -> String {
    if line.is_empty() {
        mark.to_string()
    } else {
        format!("{mark} {line}")
    }
}

/// Writes the changes as a unified diff of `file`, with [`CONTEXT`] lines
/// around each, as `git apply` and `patch` read it.
pub(super) fn write_diff(
    out: &mut dyn Write,
    file: &str,
    source: &str,
    changes: &[Change],
) -> io::Result<()> {
    let Some(_) = changes.first() else {
        return Ok(());
    };
    let old = lines(source);
    writeln!(out, "--- a/{file}")?;
    writeln!(out, "+++ b/{file}")?;
    // How many more lines the new file has than the old, before a hunk.
    let mut shift: isize = 0;
    let mut rest = changes;
    while let Some(first) = rest.first() {
        // The changes whose context touches the one before go in one hunk.
        let mut count = 1;
        while let Some(next) = rest.get(count) {
            let previous = &rest[count - 1];
            if next.line > previous.line + previous.before.len() + 2 * CONTEXT {
                break;
            }
            count += 1;
        }
        let (hunk, after) = rest.split_at(count);
        rest = after;
        let last = &hunk[hunk.len() - 1];
        let start = first.line.saturating_sub(CONTEXT);
        let end = (last.line + last.before.len() + CONTEXT).min(old.len());
        let grown: isize = hunk
            .iter()
            .map(|c| c.after.len() as isize - c.before.len() as isize)
            .sum();
        let old_count = end - start;
        let new_count = (old_count as isize + grown).max(0) as usize;
        let new_start = (start as isize + shift).max(0) as usize;
        writeln!(
            out,
            "@@ -{} +{} @@",
            range(start, old_count),
            range(new_start, new_count)
        )?;
        let mut at = start;
        for change in hunk {
            for line in &old[at..change.line] {
                writeln!(out, " {line}")?;
            }
            for line in &change.before {
                writeln!(out, "-{line}")?;
            }
            for line in &change.after {
                writeln!(out, "+{line}")?;
            }
            at = change.line + change.before.len();
        }
        for line in &old[at.min(end)..end] {
            writeln!(out, " {line}")?;
        }
        shift += grown;
    }
    Ok(())
}

/// A hunk's range as a unified diff writes it: its first line from 1 and
/// its length, or, when it's empty, the line before it.
fn range(start: usize, count: usize) -> String {
    if count == 0 {
        format!("{start},0")
    } else {
        format!("{},{count}", start + 1)
    }
}

/// The offset each line starts at.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .filter(|&s| s < text.len() || s == 0)
        .collect()
}

/// The lines of `text`, without their endings.
fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use ascribe_core::Span;

    use super::*;

    fn diff(source: &str, edits: &[TextEdit]) -> String {
        let mut out = Vec::new();
        write_diff(&mut out, "docs/a.md", source, &changes(source, edits))
            .expect("writes to memory");
        String::from_utf8(out).expect("UTF-8")
    }

    #[test]
    fn an_edit_is_shown_as_its_whole_line() {
        let source = "# Title\n\n@note{ type = tip }:Text.\n";
        let edits = [TextEdit::replace(
            Span::new(9, 34),
            "@note {type=tip}: Text.",
        )];
        assert_eq!(
            changes(source, &edits),
            [Change {
                line: 2,
                before: vec!["@note{ type = tip }:Text.".to_owned()],
                after: vec!["@note {type=tip}: Text.".to_owned()],
            }]
        );
    }

    #[test]
    fn edits_on_one_line_are_one_change() {
        let source = "@note{a=1 }:x\n";
        let edits = [
            TextEdit::insert(5, " "),
            TextEdit::delete(Span::new(9, 10)),
            TextEdit::insert(12, " "),
        ];
        let found = changes(source, &edits);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].after, ["@note {a=1}: x"]);
    }

    #[test]
    fn a_removed_line_has_nothing_after() {
        // `@steps`, a blank line, then the list: the blank line goes.
        let source = "@steps\n\n1. One\n";
        let found = changes(source, &[TextEdit::delete(Span::new(7, 8))]);
        assert_eq!(
            found,
            [Change {
                line: 1,
                before: vec![String::new()],
                after: vec![],
            }]
        );
        assert_eq!(
            diff(source, &[TextEdit::delete(Span::new(7, 8))]),
            "--- a/docs/a.md\n+++ b/docs/a.md\n@@ -1,3 +1,2 @@\n @steps\n-\n 1. One\n"
        );
    }

    #[test]
    fn the_diff_has_context_and_joins_close_changes() {
        let source: String = (1..=20).map(|n| format!("line {n}\n")).collect();
        let at = |n: usize| source.find(&format!("line {n}\n")).expect("has the line");
        let edits = [
            TextEdit::replace(Span::new(at(2), at(2) + 6), "LINE 2"),
            TextEdit::replace(Span::new(at(6), at(6) + 6), "LINE 6"),
            TextEdit::insert(at(18), "new\n"),
        ];
        let text = diff(&source, &edits);
        assert_eq!(
            text,
            "--- a/docs/a.md\n+++ b/docs/a.md\n\
             @@ -1,9 +1,9 @@\n line 1\n-line 2\n+LINE 2\n line 3\n line 4\n line 5\n-line 6\n+LINE 6\n line 7\n line 8\n line 9\n\
             @@ -15,6 +15,7 @@\n line 15\n line 16\n line 17\n-line 18\n+new\n+line 18\n line 19\n line 20\n"
        );
    }
}
