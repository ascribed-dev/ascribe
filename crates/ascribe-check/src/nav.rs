//! The navigation file, `ascribe.nav` (SPEC §7.8): the problems reading it
//! (`nav-invalid`), and, once it reads, what it says against what the
//! project has:
//!
//! - In the file, at the value each is about: a `folder` that isn't a
//!   section (`nav-folder-unknown`), a `pages` entry that isn't a page in
//!   the folder or names its `index.md` (`nav-page-unknown`), each with an
//!   unsafe fix to the closest name; and a folder listed twice, or a page
//!   named twice in one `pages`, at the second (`nav-duplicate`), with a
//!   safe fix that removes it.
//! - On a page: one a section's `pages` leaves out (`nav-page-unplaced`),
//!   and the first page of a section the file leaves out where it lists a
//!   sibling (`nav-section-unplaced`), each advice with a safe fix to the
//!   file. A section's fix appends every page it leaves out, and the
//!   file's appends every section, in the order the navigation has them
//!   now, so applying it changes no order a reader sees; each diagnostic
//!   of a section, or of the file, offers the same fix, which `--fix`
//!   applies once.
//!
//! And whether or not there's a file, a folder with an `index.md` and a
//! page beside it with its name (`section-page-both`), on the `index.md`.
//!
//! The fixes are edits to the file's text that keep its comments and
//! layout: a name in place of a name, an entry or a table taken out with
//! its own lines, and names or tables added after the last.

use std::collections::{BTreeMap, BTreeSet};

use ascribe_core::{
    Applicability, DiagnosticSlug, FileId, Fix, Issue, Location, RelPath, Span, TextEdit,
    diagnostics,
};
use ascribe_model::{Checks, NavFile, NavSection};
use ascribe_resolve::{FileKind, Navigation, Section};

use crate::across::title_span;
use crate::checks::text::{quoted_list, suggest};
use crate::project::NAV_FILE_ID;
use crate::{Diagnostic, Project};

/// What the checks found, by file: the navigation file's own under its id,
/// and the pages' under theirs, each list in source order. `index` is the
/// project's source index; `file` gives the project's id of one of its
/// files. With `errors_only`, the advice runs only when `[checks]` raises
/// it to an error.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
    errors_only: bool,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let mut issues: Vec<Issue> = Vec::new();
    both(index, file, &mut issues);
    if let Some(Err(problems)) = project.nav() {
        issues.extend(problems.iter().cloned());
    }
    if let (Some(Ok(nav)), Some(entry)) = (project.nav(), project.file(NAV_FILE_ID)) {
        let checks = &project.model().checks;
        let runs = |slug| runs(checks, slug, errors_only);
        let navigation = Navigation::of(index, Some(nav));
        let checker = Checker {
            nav,
            text: entry.text,
            index,
            file,
            sections: sections(&navigation.root),
        };
        checker.file(&mut issues);
        if runs(diagnostics::NAV_PAGE_UNPLACED) {
            checker.unplaced_pages(&mut issues);
        }
        if runs(diagnostics::NAV_SECTION_UNPLACED) {
            checker.unplaced_sections(&navigation.root, &mut issues);
        }
    }
    let mut out: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for issue in &issues {
        out.entry(issue.location.file)
            .or_default()
            .push(Diagnostic::from_issue(issue));
    }
    for list in out.values_mut() {
        list.sort_by_key(|d| d.location.span);
    }
    out
}

/// Whether a check runs: it's on and, when only errors count, `[checks]`
/// raises it to an error.
fn runs(checks: &Checks, slug: DiagnosticSlug, errors_only: bool) -> bool {
    !checks.is_off(slug)
        && (!errors_only || checks.level(slug) == Some(ascribe_model::CheckLevel::Error))
}

/// `section-page-both`: each folder with an `index.md` and the page beside
/// it with its name, on the `index.md`.
fn both(index: &ascribe_resolve::Project, file: &dyn Fn(FileId) -> FileId, out: &mut Vec<Issue>) {
    for page in index.files().filter(|f| f.kind == FileKind::Page) {
        let Some(folder) = page.path.as_str().strip_suffix("/index.md") else {
            continue;
        };
        let Some(beside) = RelPath::parse(&format!("{folder}.md")).ok() else {
            continue;
        };
        if index
            .file(&beside)
            .is_some_and(|f| f.kind == FileKind::Page)
        {
            out.push(
                Issue::new(
                    diagnostics::SECTION_PAGE_BOTH,
                    Location::new(file(page.file), title_span(page)),
                )
                .with_arg("folder", format!("{folder}/"))
                .with_arg("page", beside.as_str().to_owned()),
            );
        }
    }
}

/// Every section of the navigation, by folder.
fn sections(root: &Section) -> BTreeMap<String, &Section> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root];
    while let Some(section) = stack.pop() {
        out.insert(section.folder.clone(), section);
        stack.extend(section.sections.iter());
    }
    out
}

struct Checker<'a> {
    nav: &'a NavFile,
    text: &'a str,
    index: &'a ascribe_resolve::Project,
    file: &'a dyn Fn(FileId) -> FileId,
    sections: BTreeMap<String, &'a Section>,
}

impl Checker<'_> {
    /// The file's own problems.
    fn file(&self, out: &mut Vec<Issue>) {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for entry in &self.nav.sections {
            let at = Location::new(NAV_FILE_ID, entry.folder_span);
            if !seen.insert(&entry.folder) {
                let mut issue = Issue::new(diagnostics::NAV_DUPLICATE, at)
                    .with_arg("folder", entry.folder.clone());
                if let Some(span) = table_span(self.text, entry.folder_span) {
                    issue = issue.with_fix(fix(
                        format!("Remove the second table of `{}`", entry.folder),
                        vec![TextEdit::delete(span)],
                        Applicability::Safe,
                    ));
                }
                out.push(issue);
                continue;
            }
            let Some(section) = self.sections.get(&entry.folder) else {
                out.push(self.unknown_folder(entry, at));
                continue;
            };
            self.pages(entry, section, out);
        }
    }

    /// `nav-folder-unknown` for `entry`, with the closest section's folder.
    fn unknown_folder(&self, entry: &NavSection, at: Location) -> Issue {
        let issue = Issue::new(diagnostics::NAV_FOLDER_UNKNOWN, at)
            .with_arg("folder", entry.folder.clone());
        match closest(&entry.folder, self.sections.keys().map(String::as_str)) {
            Some(folder) => issue
                .with_variant("suggestion")
                .with_arg("suggestion", folder.to_owned())
                .with_fix(fix(
                    format!("Change the folder to `{folder}`"),
                    vec![TextEdit::replace(entry.folder_span, toml_string(folder))],
                    Applicability::Unsafe,
                )),
            None => issue,
        }
    }

    /// The problems of a listed section's `pages`.
    fn pages(&self, entry: &NavSection, section: &Section, out: &mut Vec<Issue>) {
        let Some(pages) = &entry.pages else {
            return;
        };
        let names: Vec<&str> = section.pages.iter().map(name).collect();
        let own_index = section
            .page
            .as_ref()
            .is_some_and(|p| p.as_str() == format!("{}index.md", section.folder));
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (i, (written, span)) in pages.iter().enumerate() {
            let at = Location::new(NAV_FILE_ID, *span);
            if names.contains(&written.as_str()) {
                if !seen.insert(written) {
                    let mut issue = Issue::new(diagnostics::NAV_DUPLICATE, at)
                        .with_variant("page")
                        .with_arg("folder", entry.folder.clone())
                        .with_arg("name", written.clone());
                    if let Some(span) = item_span(self.text, pages, i) {
                        issue = issue.with_fix(fix(
                            format!("Remove the second `{written}`"),
                            vec![TextEdit::delete(span)],
                            Applicability::Safe,
                        ));
                    }
                    out.push(issue);
                }
                continue;
            }
            let issue = Issue::new(diagnostics::NAV_PAGE_UNKNOWN, at)
                .with_arg("folder", entry.folder.clone())
                .with_arg("name", written.clone())
                .with_arg("values", quoted_list(&names));
            let issue = if written == "index.md" && own_index {
                issue.with_variant("own")
            } else if written.contains('/') {
                issue.with_variant("slash")
            } else if names.is_empty() {
                issue.with_variant("none")
            } else {
                match closest(written, names.iter().copied()) {
                    Some(name) => issue
                        .with_variant("suggestion")
                        .with_arg("suggestion", name.to_owned())
                        .with_fix(fix(
                            format!("Change it to `{name}`"),
                            vec![TextEdit::replace(*span, toml_string(name))],
                            Applicability::Unsafe,
                        )),
                    None => issue,
                }
            };
            out.push(issue);
        }
    }

    /// `nav-page-unplaced`, on each page of a listed section's that its
    /// `pages` leaves out.
    fn unplaced_pages(&self, out: &mut Vec<Issue>) {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for entry in &self.nav.sections {
            if !seen.insert(&entry.folder) {
                continue;
            }
            let (Some(section), Some(written), Some(list)) = (
                self.sections.get(&entry.folder),
                &entry.pages,
                entry.pages_span,
            ) else {
                continue;
            };
            let named: BTreeSet<&str> = written.iter().map(|(n, _)| n.as_str()).collect();
            // The navigation has the pages the list names first, then the
            // rest by title: the order to append them in.
            let left: Vec<&RelPath> = section
                .pages
                .iter()
                .filter(|p| !named.contains(name(p)))
                .collect();
            let Some(edit) = append_names(self.text, list, written, &left) else {
                continue;
            };
            let title = match left.as_slice() {
                [one] => format!(
                    "Add `{}` to `{}`'s pages in ascribe.nav",
                    name(one),
                    entry.folder
                ),
                _ => format!(
                    "Add the {} pages `{}`'s list leaves out to it in ascribe.nav",
                    left.len(),
                    entry.folder
                ),
            };
            for page in &left {
                let Some(at) = self.on_page(page) else {
                    continue;
                };
                out.push(
                    Issue::new(diagnostics::NAV_PAGE_UNPLACED, at)
                        .with_arg("folder", entry.folder.clone())
                        .with_fix(fix(title.clone(), vec![edit.clone()], Applicability::Safe)),
                );
            }
        }
    }

    /// `nav-section-unplaced`, on the first page of each section the file
    /// leaves out where it lists one of its siblings.
    fn unplaced_sections(&self, root: &Section, out: &mut Vec<Issue>) {
        let mut left: Vec<(&Section, &str)> = Vec::new();
        let mut stack = vec![root];
        while let Some(parent) = stack.pop() {
            if parent.sections.iter().any(|s| s.listed) {
                left.extend(
                    parent
                        .sections
                        .iter()
                        .filter(|s| !s.listed)
                        .map(|s| (s, parent.folder.as_str())),
                );
            }
            stack.extend(parent.sections.iter());
        }
        if left.is_empty() {
            return;
        }
        // Appended in path order, each comes after the listed ones among its
        // siblings, as it does now.
        left.sort_by(|a, b| a.0.folder.cmp(&b.0.folder));
        let mut tables = String::new();
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            tables.push('\n');
        }
        for (section, _) in &left {
            tables.push_str(&format!(
                "\n[[sections]]\nfolder = {}\n",
                toml_string(&section.folder)
            ));
        }
        let edit = TextEdit::insert(self.text.len(), tables);
        let title = match left.as_slice() {
            [(one, _)] => format!("Add `{}` to ascribe.nav", one.folder),
            _ => format!(
                "Add the {} sections ascribe.nav leaves out to it",
                left.len()
            ),
        };
        for (section, parent) in &left {
            let Some(at) = first_page(section).and_then(|p| self.on_page(p)) else {
                continue;
            };
            let issue = Issue::new(diagnostics::NAV_SECTION_UNPLACED, at)
                .with_arg("folder", section.folder.clone())
                .with_arg("parent", (*parent).to_owned());
            let issue = if parent.is_empty() {
                issue.with_variant("root")
            } else {
                issue
            };
            out.push(issue.with_fix(fix(title.clone(), vec![edit.clone()], Applicability::Safe)));
        }
    }

    /// Where a problem about a page is reported: at its title.
    fn on_page(&self, page: &RelPath) -> Option<Location> {
        let indexed = self.index.file(page)?;
        Some(Location::new(
            (self.file)(indexed.file),
            title_span(indexed),
        ))
    }
}

/// A fix to the navigation file.
fn fix(title: String, edits: Vec<TextEdit>, applicability: Applicability) -> Fix {
    Fix {
        title,
        file: NAV_FILE_ID,
        edits,
        applicability,
    }
}

/// A section's first page in reading order: its own, else its first, else
/// its first nested section's.
fn first_page(section: &Section) -> Option<&RelPath> {
    section
        .page
        .as_ref()
        .or_else(|| section.pages.first())
        .or_else(|| section.sections.iter().find_map(first_page))
}

/// A page's file name.
fn name(page: &RelPath) -> &str {
    page.as_str().rsplit('/').next().unwrap_or_default()
}

/// The closest of `candidates` to `written`: one that differs only in case,
/// else one close enough to be a typo.
fn closest<'a>(
    written: &str,
    candidates: impl Iterator<Item = &'a str> + Clone,
) -> Option<&'a str> {
    let lower = written.to_lowercase();
    candidates
        .clone()
        .find(|c| c.to_lowercase() == lower)
        .or_else(|| suggest(written, candidates))
}

/// A TOML basic string of `text`.
fn toml_string(text: &str) -> String {
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
fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// The end of the line `at` is on, after its line break.
fn line_end(text: &str, at: usize) -> usize {
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
fn table_span(text: &str, folder: Span) -> Option<Span> {
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

/// The text to take out with the `i`th of a `pages` list: its own lines
/// when it starts one, else from the end of the entry before it.
fn item_span(text: &str, pages: &[(String, Span)], i: usize) -> Option<Span> {
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

/// The edit that appends `left`'s names to a `pages` list at `list`, whose
/// entries are `written`, in the list's own layout: one a line when it's
/// written so, else on its line.
fn append_names(
    text: &str,
    list: Span,
    written: &[(String, Span)],
    left: &[&RelPath],
) -> Option<TextEdit> {
    if left.is_empty() {
        return None;
    }
    let names: Vec<String> = left.iter().map(|p| toml_string(name(p))).collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use ascribe_core::apply_edits;

    fn names(list: &[&str]) -> Vec<RelPath> {
        list.iter()
            .map(|n| RelPath::parse(n).expect("a path"))
            .collect()
    }

    /// `text` with `left` appended to its only section's `pages`.
    fn appended(text: &str, left: &[&str]) -> String {
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        let entry = &nav.sections[0];
        let left = names(left);
        let left: Vec<&RelPath> = left.iter().collect();
        let edit = append_names(
            text,
            entry.pages_span.expect("a list"),
            entry.pages.as_deref().expect("pages"),
            &left,
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

    #[test]
    fn a_second_entry_is_taken_out_with_its_own_text() {
        let take = |text: &str, i: usize| {
            let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
            let pages = nav.sections[0].pages.clone().expect("pages");
            let span = item_span(text, &pages, i).expect("a span");
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
    fn a_second_table_is_taken_out_leaving_the_next_ones_comments() {
        let text = "version = 1\n\n[[sections]]\nfolder = \"a/\"\n\n[[sections]]\nfolder = \"a/\"\nlabel = \"A\"\n\n# The guides.\n[[sections]]\nfolder = \"b/\"\n";
        let nav = NavFile::parse(text, FileId::new(0)).expect("it reads");
        let span = table_span(text, nav.sections[1].folder_span).expect("a table");
        assert_eq!(
            apply_edits(text, &[TextEdit::delete(span)]).expect("it applies"),
            "version = 1\n\n[[sections]]\nfolder = \"a/\"\n\n# The guides.\n[[sections]]\nfolder = \"b/\"\n"
        );
        let last = "version = 1\n[[sections]]\nfolder = \"a/\"\n[[sections]]\nfolder = \"a/\"\n";
        let nav = NavFile::parse(last, FileId::new(0)).expect("it reads");
        let span = table_span(last, nav.sections[1].folder_span).expect("a table");
        assert_eq!(
            apply_edits(last, &[TextEdit::delete(span)]).expect("it applies"),
            "version = 1\n[[sections]]\nfolder = \"a/\"\n"
        );
        let inline = "version = 1\nsections = [{ folder = \"a/\" }, { folder = \"a/\" }]\n";
        let nav = NavFile::parse(inline, FileId::new(0)).expect("it reads");
        assert_eq!(table_span(inline, nav.sections[1].folder_span), None);
    }

    #[test]
    fn strings_are_toml() {
        assert_eq!(toml_string("My Setup.md"), "\"My Setup.md\"");
        assert_eq!(toml_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }

    #[test]
    fn closest_prefers_a_difference_of_case() {
        assert_eq!(
            closest("options.md", ["Options.md", "option.md"].into_iter()),
            Some("Options.md")
        );
        assert_eq!(
            closest("Optoins.md", ["Options.md"].into_iter()),
            Some("Options.md")
        );
        assert_eq!(closest("zzz.md", ["Options.md"].into_iter()), None);
    }
}
