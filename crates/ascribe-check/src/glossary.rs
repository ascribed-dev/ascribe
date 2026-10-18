//! The glossary against the content (SPEC §5.4):
//!
//! - `glossary-term-duplicate`: two terms, or a term and an alias, with the
//!   same text, or two terms with the same id, reported at the second,
//!   naming the first. Matching, and a link to the id, can't choose between
//!   them, so the second is left out of the glossary.
//! - `glossary-definition-missing`: a term whose section doesn't start with a
//!   paragraph, at its heading.
//! - `term-off-glossary-page`: `@term` on a heading no glossary page shows,
//!   at the directive. On a page, its fix lists the page in `[glossary]
//!   pages`; the fix is unsafe, since a listed page's terms are linked on
//!   every other page.
//!
//! They hold for every build, since a term is the source's: its heading's
//! text, its `@term`, and the block under it don't depend on a build.

use std::collections::BTreeMap;

use ascribe_core::{Applicability, FileId, Fix, Issue, Location, TextEdit, diagnostics};
use toml_edit::{Document, Item, Value};

use crate::{Diagnostic, Project};

/// What the checks found, by file, in source order. `index` is the
/// project's source index; `file` gives the project's id of one of its
/// files.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let glossary = index.glossary();
    let mut issues = Vec::new();
    for duplicate in glossary.duplicates {
        let mut issue = Issue::new(
            diagnostics::GLOSSARY_TERM_DUPLICATE,
            Location::new(file(duplicate.file), duplicate.span),
        )
        .with_arg("text", duplicate.text)
        .with_arg("first", duplicate.first)
        .with_arg("page", duplicate.first_page.to_string());
        if duplicate.id {
            issue = issue.with_variant("id");
        }
        issues.push(issue);
    }
    for term in glossary.terms.iter().filter(|t| t.definition.is_empty()) {
        issues.push(
            Issue::new(
                diagnostics::GLOSSARY_DEFINITION_MISSING,
                Location::new(file(term.file), term.heading),
            )
            .with_arg("term", term.term.clone()),
        );
    }
    for off in glossary.off_page {
        let path = off.path.to_string();
        let mut issue = Issue::new(
            diagnostics::TERM_OFF_GLOSSARY_PAGE,
            Location::new(file(off.file), off.mark),
        )
        .with_arg("path", path.clone());
        if !off.page {
            issue = issue.with_variant("fragment");
        } else if let Some(edit) = add_page(project.model_text(), &path) {
            issue = issue.with_fix(Fix {
                title: format!("List `{path}` in `[glossary] pages`"),
                file: FileId::new(0),
                edits: vec![edit],
                // Every term on the page is linked on every other page.
                applicability: Applicability::Unsafe,
            });
        }
        issues.push(issue);
    }
    let mut out: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for issue in issues {
        out.entry(issue.location.file)
            .or_default()
            .push(Diagnostic::from_issue(&issue));
    }
    for list in out.values_mut() {
        list.sort_by_key(|d| d.location.span);
    }
    out
}

/// The edit to `ascribe.toml`'s `text` that adds `path` to `[glossary]
/// pages`, keeping everything else as it's written: the path after the
/// array's last item, `pages` after the table's last key, or the table at
/// the end of the file. `None` when the file isn't valid TOML, or writes
/// the glossary in a form an edit can't extend in place (an inline table, a
/// dotted key), or the edited file wouldn't hold the path.
pub(crate) fn add_page(text: &str, path: &str) -> Option<TextEdit> {
    let doc = Document::parse(text).ok()?;
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let quoted = Value::from(path).to_string();
    let quoted = quoted.trim();
    let edit = match doc.get("glossary") {
        None => {
            let lead = if text.is_empty() || text.ends_with('\n') {
                nl.to_owned()
            } else {
                format!("{nl}{nl}")
            };
            TextEdit::insert(
                text.len(),
                format!("{lead}[glossary]{nl}pages = [{quoted}]{nl}"),
            )
        }
        Some(Item::Table(table)) if !table.is_implicit() && !table.is_dotted() => {
            match table.get("pages") {
                Some(Item::Value(Value::Array(array))) => match array.iter().last() {
                    Some(last) => TextEdit::insert(last.span()?.end, format!(", {quoted}")),
                    None => TextEdit::insert(array.span()?.start + 1, quoted.to_owned()),
                },
                None => {
                    // After the table's last entry, or its header.
                    let after = table
                        .iter()
                        .filter_map(|(_, item)| item.span())
                        .map(|span| span.end)
                        .chain(table.span().map(|span| span.end))
                        .max()?;
                    match text.get(after..)?.find('\n') {
                        Some(i) => {
                            TextEdit::insert(after + i + 1, format!("pages = [{quoted}]{nl}"))
                        }
                        None => TextEdit::insert(text.len(), format!("{nl}pages = [{quoted}]")),
                    }
                }
                Some(_) => return None,
            }
        }
        Some(_) => return None,
    };
    let mut edited = text.to_owned();
    edited.replace_range(edit.span.range(), &edit.new_text);
    let lists = edited
        .parse::<toml::Table>()
        .ok()?
        .get("glossary")?
        .get("pages")?
        .as_array()?
        .iter()
        .any(|p| p.as_str() == Some(path));
    lists.then_some(edit)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn added(text: &str) -> Option<String> {
        let edit = add_page(text, "guide/terms.md")?;
        let mut out = text.to_owned();
        out.replace_range(edit.span.range(), &edit.new_text);
        Some(out)
    }

    #[test]
    fn the_page_goes_after_the_last_listed() {
        assert_eq!(
            added("[glossary]\n# The terms.\npages = [\n  \"glossary.md\", # first\n]\n# After.\n")
                .unwrap(),
            "[glossary]\n# The terms.\npages = [\n  \"glossary.md\", \"guide/terms.md\", # first\n]\n# After.\n"
        );
        assert_eq!(
            added("[glossary]\npages = []\n").unwrap(),
            "[glossary]\npages = [\"guide/terms.md\"]\n"
        );
    }

    #[test]
    fn pages_goes_after_the_tables_last_key() {
        assert_eq!(
            added("spec = \"0.1\"\n\n[glossary] # terms\nmatch = \"every\" # all\n\n[phrases]\nx = \"y\"\n")
                .unwrap(),
            "spec = \"0.1\"\n\n[glossary] # terms\nmatch = \"every\" # all\npages = [\"guide/terms.md\"]\n\n[phrases]\nx = \"y\"\n"
        );
        assert_eq!(
            added("[glossary]").unwrap(),
            "[glossary]\npages = [\"guide/terms.md\"]"
        );
    }

    #[test]
    fn the_table_goes_at_the_end() {
        assert_eq!(
            added("spec = \"0.1\" # the spec\n").unwrap(),
            "spec = \"0.1\" # the spec\n\n[glossary]\npages = [\"guide/terms.md\"]\n"
        );
        assert_eq!(
            added("spec = \"0.1\"").unwrap(),
            "spec = \"0.1\"\n\n[glossary]\npages = [\"guide/terms.md\"]\n"
        );
    }

    #[test]
    fn a_glossary_written_inline_or_dotted_is_left_alone() {
        assert_eq!(added("glossary = { match = \"every\" }\n"), None);
        assert_eq!(added("glossary.match = \"every\"\n"), None);
        assert_eq!(added("[glossary]\npages = \"glossary.md\"\n"), None);
        assert_eq!(added("[glossary\n"), None);
    }
}
