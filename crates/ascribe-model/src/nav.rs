//! `ascribe.nav` (SPEC §7.8): the order a project's sections and pages come
//! in, and their labels, which every sidebar is drawn from.
//!
//! A section is a folder of the content root, and the file lists some of
//! them, each at most once, in order: `folder`, a path from the content root
//! ending in `/` (`""` for the root), and optionally `pages`, the file names
//! of its pages in order, and `label`. What it leaves out has a default
//! order, which `ascribe_resolve::navigation` works out.
//!
//! [`NavFile::parse`] reads the file. A file that can't be read as that
//! grammar is `nav-invalid`, and the project has the default order, as if it
//! had no file. A `folder` or a `pages` entry that names nothing is passed
//! over here; it's for the checks to report.

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};

use toml::de::{DeTable, DeValue};

use crate::toml_util::{describe, entries, sp};

/// The file's name, beside `ascribe.toml`.
pub const NAV_FILE: &str = "ascribe.nav";

/// The file's format, its `version`.
pub const NAV_VERSION: i64 = 1;

/// What the file says: its sections, in the order it lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavFile {
    /// Each `[[sections]]` entry, in order.
    pub sections: Vec<NavSection>,
}

/// One `[[sections]]` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavSection {
    /// The folder, from the content root, ending in `/`; `""` for the root.
    pub folder: String,
    /// Where `folder`'s value is.
    pub folder_span: Span,
    /// The file names of its pages, in order, each with its span; `None`
    /// when the entry has no `pages`.
    pub pages: Option<Vec<(String, Span)>>,
    /// Where `pages`' list is, brackets included, when it's one.
    pub pages_span: Option<Span>,
    /// Its label, when it sets one.
    pub label: Option<String>,
}

impl NavFile {
    /// Reads the file's text. Issues are `nav-invalid`, located in `file`;
    /// with any, the file isn't used.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str, file: FileId) -> Result<NavFile, Vec<Issue>> {
        let mut reader = Reader {
            file,
            issues: Vec::new(),
        };
        let nav = reader.file(text);
        if reader.issues.is_empty() {
            Ok(nav)
        } else {
            Err(reader.issues)
        }
    }

    /// The entry for `folder`: the first, when the file lists it twice.
    pub fn section(&self, folder: &str) -> Option<&NavSection> {
        self.sections.iter().find(|s| s.folder == folder)
    }

    /// Where `folder` is among the sections the file lists, counting each
    /// folder once, by its first entry.
    pub fn position(&self, folder: &str) -> Option<usize> {
        self.sections.iter().position(|s| s.folder == folder)
    }
}

struct Reader {
    file: FileId,
    issues: Vec<Issue>,
}

impl Reader {
    fn problem(&mut self, span: Span, detail: impl Into<String>) {
        self.issues.push(
            Issue::new(diagnostics::NAV_INVALID, Location::new(self.file, span))
                .with_arg("detail", detail.into()),
        );
    }

    fn file(&mut self, text: &str) -> NavFile {
        let mut nav = NavFile::default();
        let parsed = match DeTable::parse(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                let span = e
                    .span()
                    .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
                self.problem(span, e.message().trim().to_owned());
                return nav;
            }
        };
        let root = parsed.get_ref();
        for (key, span, _) in entries(root) {
            if !matches!(key, "version" | "sections") {
                self.problem(span, format!("it has no key `{key}`"));
            }
        }
        match root.get("version") {
            None => self.problem(Span::empty(0), "it has no `version`"),
            Some(v) => match v.get_ref() {
                DeValue::Integer(i) if i.as_str() == NAV_VERSION.to_string() => {}
                DeValue::Integer(i) => self.problem(
                    sp(v),
                    format!(
                        "it's version {}, and this version of Ascribe reads version {NAV_VERSION}",
                        i.as_str()
                    ),
                ),
                other => self.problem(
                    sp(v),
                    format!("`version` is {}, not a number", describe(other)),
                ),
            },
        }
        let Some(sections) = root.get("sections") else {
            return nav;
        };
        let DeValue::Array(list) = sections.get_ref() else {
            self.problem(
                sp(sections),
                format!(
                    "`sections` is {}, not a list of `[[sections]]` tables",
                    describe(sections.get_ref())
                ),
            );
            return nav;
        };
        for item in list.iter() {
            match item.get_ref() {
                DeValue::Table(table) => {
                    if let Some(section) = self.section(table, sp(item)) {
                        nav.sections.push(section);
                    }
                }
                other => self.problem(
                    sp(item),
                    format!("a section is {}, not a table", describe(other)),
                ),
            }
        }
        nav
    }

    fn section(&mut self, table: &DeTable<'_>, span: Span) -> Option<NavSection> {
        for (key, key_span, _) in entries(table) {
            if !matches!(key, "folder" | "pages" | "label") {
                self.problem(key_span, format!("a section has no key `{key}`"));
            }
        }
        let folder = match table.get("folder") {
            None => {
                self.problem(span, "a section has no `folder`");
                None
            }
            Some(v) => match v.get_ref() {
                DeValue::String(text) if is_folder(text) => Some((text.to_string(), sp(v))),
                DeValue::String(text) => {
                    self.problem(
                        sp(v),
                        format!(
                            "the folder `{text}` isn't a folder's path from the content root ending in `/`, or `\"\"` for the root"
                        ),
                    );
                    None
                }
                other => {
                    self.problem(
                        sp(v),
                        format!("`folder` is {}, not a string", describe(other)),
                    );
                    None
                }
            },
        };
        let pages_span = table
            .get("pages")
            .filter(|v| matches!(v.get_ref(), DeValue::Array(_)))
            .map(sp);
        let pages = table.get("pages").and_then(|v| match v.get_ref() {
            DeValue::Array(list) => {
                let mut names = Vec::new();
                for item in list.iter() {
                    match item.get_ref() {
                        DeValue::String(name) => names.push((name.to_string(), sp(item))),
                        other => self.problem(
                            sp(item),
                            format!("a page in `pages` is {}, not a file name", describe(other)),
                        ),
                    }
                }
                Some(names)
            }
            other => {
                self.problem(
                    sp(v),
                    format!("`pages` is {}, not a list of file names", describe(other)),
                );
                None
            }
        });
        let label = table.get("label").and_then(|v| match v.get_ref() {
            DeValue::String(text) if text.trim().is_empty() => {
                self.problem(sp(v), "`label` is empty");
                None
            }
            DeValue::String(text) if text.contains(['\n', '\r']) => {
                self.problem(sp(v), "`label` is more than one line");
                None
            }
            DeValue::String(text) => Some(text.to_string()),
            other => {
                self.problem(
                    sp(v),
                    format!("`label` is {}, not a string", describe(other)),
                );
                None
            }
        });
        let (folder, folder_span) = folder?;
        Some(NavSection {
            folder,
            folder_span,
            pages,
            pages_span,
            label,
        })
    }
}

/// A folder as the file writes it: `""`, or segments ending in `/`, none
/// empty, `.`, or `..`, with no `\`.
fn is_folder(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    let Some(path) = text.strip_suffix('/') else {
        return false;
    };
    !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> NavFile {
        NavFile::parse(text, FileId::new(0)).expect("it reads")
    }

    fn bad(text: &str) -> Vec<String> {
        NavFile::parse(text, FileId::new(0))
            .expect_err(text)
            .iter()
            .map(|i| i.args.iter().map(|a| a.value.clone()).collect::<String>())
            .collect()
    }

    #[test]
    fn sections_in_order() {
        let nav = read(
            "version = 1\n\n[[sections]]\nfolder = \"\"\n\n[[sections]]\nfolder = \"Reference/\"\nlabel = \"API\"\npages = [\"Options.md\", \"index.md\"]\n\n[[sections]]\nfolder = \"Guides/My Setup/\"\n",
        );
        let folders: Vec<&str> = nav.sections.iter().map(|s| s.folder.as_str()).collect();
        assert_eq!(folders, ["", "Reference/", "Guides/My Setup/"]);
        let reference = nav.section("Reference/").expect("listed");
        assert_eq!(reference.label.as_deref(), Some("API"));
        let pages: Vec<&str> = reference
            .pages
            .iter()
            .flatten()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(pages, ["Options.md", "index.md"]);
        assert_eq!(nav.section("").and_then(|s| s.pages.as_ref()), None);
        assert_eq!(nav.position("Guides/My Setup/"), Some(2));
        let text = "version = 1\n[[sections]]\nfolder = \"a/\"\npages = [ \"b.md\" ]\n";
        let nav = read(text);
        let a = &nav.sections[0];
        assert_eq!(&text[a.folder_span.range()], "\"a/\"");
        assert_eq!(a.pages_span.map(|s| &text[s.range()]), Some("[ \"b.md\" ]"));
        assert_eq!(
            a.pages.as_ref().map(|p| &text[p[0].1.range()]),
            Some("\"b.md\"")
        );
        assert!(read("version = 1\n").sections.is_empty());
    }

    #[test]
    fn problems() {
        assert_eq!(
            bad("version = 2\n"),
            ["it's version 2, and this version of Ascribe reads version 1"]
        );
        assert_eq!(bad(""), ["it has no `version`"]);
        assert!(bad("version = ")[0].contains("string"));
        assert_eq!(
            bad("version = \"1\"\n"),
            ["`version` is a string, not a number"]
        );
        assert_eq!(bad("version = 1\norder = 1\n"), ["it has no key `order`"]);
        assert_eq!(
            bad("version = 1\nsections = 1\n"),
            ["`sections` is a number, not a list of `[[sections]]` tables"]
        );
        assert_eq!(
            bad("version = 1\nsections = [1]\n"),
            ["a section is a number, not a table"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nlabel = \"A\"\n"),
            ["a section has no `folder`"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = \"\"\nweight = 1\n"),
            ["a section has no key `weight`"]
        );
        for folder in ["guides", "/guides/", "a//", "../a/", "a\\\\b/"] {
            assert_eq!(
                bad(&format!(
                    "version = 1\n[[sections]]\nfolder = \"{folder}\"\n"
                ))
                .len(),
                1,
                "{folder}"
            );
        }
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = 1\n"),
            ["`folder` is a number, not a string"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = \"\"\npages = \"a.md\"\n"),
            ["`pages` is a string, not a list of file names"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = \"\"\npages = [true]\n"),
            ["a page in `pages` is a boolean, not a file name"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = \"\"\nlabel = \" \"\n"),
            ["`label` is empty"]
        );
        assert_eq!(
            bad("version = 1\n[[sections]]\nfolder = \"\"\nlabel = \"a\\nb\"\n"),
            ["`label` is more than one line"]
        );
    }

    #[test]
    fn names_that_match_nothing_are_kept_for_the_checks() {
        // A `pages` entry with a `/`, or the same folder twice, is read; the
        // navigation passes over what names nothing.
        let nav = read(
            "version = 1\n[[sections]]\nfolder = \"a/\"\npages = [\"b/c.md\"]\n[[sections]]\nfolder = \"a/\"\n",
        );
        assert_eq!(nav.sections.len(), 2);
        assert_eq!(nav.position("a/"), Some(0));
    }
}
