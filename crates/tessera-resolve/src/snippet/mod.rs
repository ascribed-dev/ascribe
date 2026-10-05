//! Snippets (SPEC §4.8): code examples a page takes from files outside its
//! content, through the sources the content model declares (SPEC §7.3).
//!
//! This is the one implementation of the rules for snippets, as
//! [`crate::references`] is for links: the source index ([`crate::Project`])
//! and the file-level checks (`tessera-check`) both call it, so `ascribe
//! check`, the build, and the language server can't disagree about what a
//! snippet is. It covers:
//!
//! - reading an address, `<source>:<path>#<region>` ([`parse_address`]);
//! - finding its file through its source, with the source's patterns and
//!   exact-case names ([`resolve_snippet`]), and reading each file once per
//!   [`CodeFiles`];
//! - the tags in the file and the snippet they mark ([`tags`]);
//! - the code block it becomes ([`Snippet`]), and the file-level issues when
//!   it can't be one ([`snippet_issues`]).

pub mod tags;

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use tessera_core::{FileId, Issue, Location, RelPath, Span, diagnostics, percent_decode};
use tessera_model::{ContentModel, suggest};
use tessera_syntax::{DirectiveLine, PrimaryValue};

use crate::fs::{FileSystem, Probe};
use tags::{CommentStyle, Extracted, TagProblem, Tags, comment_style, is_region_name};

/// The file ids code files get, from this one up: past any source file's, so
/// a diagnostic's related location can point into a code file.
pub const CODE_FILE_IDS: u32 = 0x8000_0000;

/// A snippet's address (SPEC §4.8): `<source>:<path>`, with an optional
/// `#<region>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address {
    /// The source's name.
    pub source: String,
    /// The file's path relative to the source's folder, percent-decoded.
    pub path: String,
    /// The region, if the address names one.
    pub region: Option<String>,
}

impl Address {
    /// The file the address names, without the region: `<source>:<path>`.
    pub fn file(&self) -> String {
        format!("{}:{}", self.source, self.path)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.source, self.path)?;
        if let Some(region) = &self.region {
            write!(f, "#{region}")?;
        }
        Ok(())
    }
}

/// Why a primary isn't an address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressError {
    /// It names no source: there's no `:`, as in a relative path.
    NoSource,
    /// It has a source, but isn't well formed; the text says how.
    Invalid(&'static str),
}

/// Reads a `@snippet` primary as an address (SPEC Appendix A
/// `snippet-address`).
pub fn parse_address(primary: &str) -> Result<Address, AddressError> {
    let Some((source, rest)) = primary.split_once(':') else {
        return Err(AddressError::NoSource);
    };
    if !is_key(source) {
        return Err(AddressError::Invalid(
            "the source's name must be a lowercase letter, then lowercase letters, digits, or hyphens",
        ));
    }
    let (path, region) = match rest.split_once('#') {
        Some((path, region)) => (path, Some(region)),
        None => (rest, None),
    };
    if path.is_empty() {
        return Err(AddressError::Invalid("there's no path after the source"));
    }
    let path = percent_decode(path);
    if path.starts_with('/') {
        return Err(AddressError::Invalid("the path can't start with `/`"));
    }
    for segment in path.split('/') {
        match segment {
            "" => return Err(AddressError::Invalid("the path has an empty segment")),
            "." | ".." => {
                return Err(AddressError::Invalid(
                    "the path can't have `.` or `..` segments: it's relative to the source's folder, and stays inside it",
                ));
            }
            s if s.contains('\0') || s.contains('\\') => {
                return Err(AddressError::Invalid(
                    "the path can't contain `\\` or NUL; separate folders with `/`",
                ));
            }
            _ => {}
        }
    }
    let region = match region {
        None => None,
        Some("") => return Err(AddressError::Invalid("there's no region name after `#`")),
        Some(r) if !is_region_name(r) => {
            return Err(AddressError::Invalid(
                "a region name is letters, digits, `-`, `_`, and `.`",
            ));
        }
        Some(r) => Some(r.to_owned()),
    };
    Ok(Address {
        source: source.to_owned(),
        path,
        region,
    })
}

/// SPEC Appendix A `key`.
fn is_key(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A `@snippet` directive (SPEC §4.8), as the index records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetUse {
    /// The directive line.
    pub span: Span,
    /// The primary, when there is one (the parser reports a missing one).
    pub primary: Option<Span>,
    /// The primary as written.
    pub written: String,
    /// The primary read as an address; `None` without a primary.
    pub address: Option<Result<Address, AddressError>>,
    /// The `lang` attribute.
    pub lang: Option<String>,
    /// The `title` attribute.
    pub title: Option<String>,
    /// Whether `phrases=true`.
    pub phrases: bool,
}

impl SnippetUse {
    /// Reads a `@snippet` directive line.
    pub fn of(line: &DirectiveLine) -> SnippetUse {
        let attribute = |key: &str| {
            line.attributes
                .as_ref()
                .and_then(|a| a.get(key))
                .and_then(|a| a.value.as_ref())
                .and_then(|v| v.as_text())
                .map(str::to_owned)
        };
        let (primary, written) = match &line.primary {
            Some(PrimaryValue::Identifier(p)) => (Some(p.span), Some(p.text.clone())),
            Some(other) => (Some(other.span()), None),
            None => (None, None),
        };
        SnippetUse {
            span: line.span,
            primary,
            address: written.as_deref().map(parse_address),
            written: written.unwrap_or_default(),
            lang: attribute("lang"),
            title: attribute("title"),
            phrases: attribute("phrases").as_deref() == Some("true"),
        }
    }

    /// The fenced code block's info string (SPEC §4.8): the language, then
    /// `title="…"`, then `phrases=true`. With no language but other words,
    /// it starts with `text`, so no word is read as the language.
    pub fn info(&self, file: &CodeFile) -> String {
        let lang = self
            .lang
            .clone()
            .or_else(|| file.path.extension().map(str::to_ascii_lowercase));
        let mut words: Vec<String> = Vec::new();
        if let Some(title) = &self.title {
            let escaped = title.replace('\\', "\\\\").replace('"', "\\\"");
            words.push(format!("title=\"{escaped}\""));
        }
        if self.phrases {
            words.push("phrases=true".to_owned());
        }
        match lang {
            Some(lang) => words.insert(0, lang),
            None if !words.is_empty() => words.insert(0, "text".to_owned()),
            None => {}
        }
        words.join(" ")
    }
}

/// A code file a snippet reads, through a source.
#[derive(Debug)]
pub struct CodeFile {
    /// Its id, from [`CODE_FILE_IDS`] up, for locations in it.
    pub id: FileId,
    /// Its path relative to the project root, which may start with `..`.
    pub path: RelPath,
    /// Its text.
    pub text: String,
    /// Its comment syntax, by extension; `None` when it has no tags.
    pub style: Option<CommentStyle>,
    /// Its tags.
    pub tags: Tags,
}

impl CodeFile {
    /// Takes out the snippet of a region, or of the whole file.
    pub fn extract(&self, region: Option<&str>) -> Option<Extracted> {
        let region = match region {
            Some(name) => Some(self.tags.region(name)?),
            None => None,
        };
        Some(tags::extract(&self.text, &self.tags, region))
    }

    /// The names of its regions, in the order they start.
    pub fn region_names(&self) -> Vec<&str> {
        self.tags.regions.iter().map(|r| r.name.as_str()).collect()
    }
}

/// Why a file couldn't be read as a code file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// No file has this exact name.
    Missing,
    /// A file whose name differs only in case does: its path from the
    /// project root.
    Case(RelPath),
    /// It isn't text; the reason says why.
    NotText(String),
}

/// The code files read so far, by path from the project root, so each is
/// read and scanned once however many snippets use it. Each new file gets
/// the next id from [`CODE_FILE_IDS`].
#[derive(Debug, Default)]
pub struct CodeFiles {
    files: Mutex<HashMap<RelPath, Result<Arc<CodeFile>, ReadError>>>,
}

impl CodeFiles {
    /// No files read yet.
    pub fn new() -> CodeFiles {
        CodeFiles::default()
    }

    /// The code file at `path`, from the project root, read through `fs` the
    /// first time it's asked for.
    pub fn read(&self, fs: &dyn FileSystem, path: &RelPath) -> Result<Arc<CodeFile>, ReadError> {
        let mut files = self.files.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(found) = files.get(path) {
            return found.clone();
        }
        let next = files.values().filter(|f| f.is_ok()).count() as u32;
        let read = read_code(fs, path, FileId::new(CODE_FILE_IDS + next));
        files.insert(path.clone(), read.clone());
        read
    }

    /// The code file with this id, if it has been read.
    pub fn by_id(&self, id: FileId) -> Option<Arc<CodeFile>> {
        let files = self.files.lock().unwrap_or_else(PoisonError::into_inner);
        files
            .values()
            .filter_map(|f| f.as_ref().ok())
            .find(|f| f.id == id)
            .cloned()
    }

    /// Every code file read, in id order.
    pub fn all(&self) -> Vec<Arc<CodeFile>> {
        let files = self.files.lock().unwrap_or_else(PoisonError::into_inner);
        let mut out: Vec<Arc<CodeFile>> = files.values().filter_map(|f| f.clone().ok()).collect();
        out.sort_by_key(|f| f.id);
        out
    }
}

fn read_code(fs: &dyn FileSystem, path: &RelPath, id: FileId) -> Result<Arc<CodeFile>, ReadError> {
    match fs.probe(path) {
        Probe::File => {}
        Probe::Missing => return Err(ReadError::Missing),
        Probe::CaseMismatch(actual) => return Err(ReadError::Case(actual)),
    }
    let bytes = fs
        .read_file(path)
        .map_err(|e| ReadError::NotText(e.to_string()))?;
    if bytes.contains(&0) {
        return Err(ReadError::NotText("it has a NUL character".to_owned()));
    }
    let text =
        String::from_utf8(bytes).map_err(|_| ReadError::NotText("it isn't UTF-8".to_owned()))?;
    let style = path.extension().and_then(comment_style);
    let tags = style.map(|s| tags::scan(&text, s)).unwrap_or_default();
    Ok(Arc::new(CodeFile {
        id,
        path: path.clone(),
        text,
        style,
        tags,
    }))
}

/// A snippet: its file, and the code it takes from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    /// The address, as written.
    pub address: String,
    /// The code file's path from the project root.
    pub path: RelPath,
    /// The code file's id.
    pub file: FileId,
    /// The code block's info string.
    pub info: String,
    /// The code, each line ending in `\n`.
    pub code: String,
    /// The lines of the code file it covers, from 1; `None` when they're
    /// none.
    pub lines: Option<(u32, u32)>,
}

/// Why a snippet's address doesn't give a snippet.
#[derive(Clone, Debug)]
pub enum SnippetError {
    /// The content model declares no such source.
    UnknownSource,
    /// The file isn't there, or isn't there with this exact name.
    Missing {
        /// The path, relative to the source's folder, of a file whose name
        /// differs only in case.
        actual: Option<String>,
    },
    /// The file is there, but the source's patterns don't make it readable.
    NotIncluded,
    /// The file is a link, or is in a linked folder, that leads out of the
    /// source's folder or to a file its patterns don't make readable.
    Link,
    /// The file isn't text.
    NotText(String),
    /// The file's tags have problems.
    Tags(Arc<CodeFile>),
    /// The file has no such region.
    RegionMissing(Arc<CodeFile>),
}

/// The snippet `snippet` names, read through `fs` and the model's sources,
/// or why there's none. `code` holds the files read so far.
pub fn resolve_snippet(
    snippet: &SnippetUse,
    address: &Address,
    model: &ContentModel,
    fs: &dyn FileSystem,
    code: &CodeFiles,
) -> Result<Snippet, SnippetError> {
    let source = model
        .source(&address.source)
        .ok_or(SnippetError::UnknownSource)?;
    let folder =
        RelPath::parse(&source.path).map_err(|_| SnippetError::Missing { actual: None })?;
    let path = folder
        .join(&address.path)
        .map_err(|_| SnippetError::Missing { actual: None })?;
    // A file the source doesn't make readable isn't read at all. A source in
    // another repository has only the copies of files it does, so there's no
    // telling whether the file exists.
    if !source.reads(&address.path) {
        return Err(match fs.probe(&path) {
            Probe::Missing if source.git.is_none() => SnippetError::Missing { actual: None },
            _ => SnippetError::NotIncluded,
        });
    }
    // The file is read where its links lead, so that must be in the source
    // too: a link can't step over the folder, the patterns, or the
    // repository's edge.
    if fs.probe(&path) == Probe::File {
        let inside = fs
            .real_path(&path)
            .zip(fs.real_path(&folder))
            .and_then(|(real, folder)| relative_to(&real, &folder))
            .is_some_and(|rest| source.reads(&rest));
        if !inside {
            return Err(SnippetError::Link);
        }
    }
    let file = code.read(fs, &path).map_err(|e| match e {
        ReadError::Missing => SnippetError::Missing { actual: None },
        ReadError::Case(actual) => SnippetError::Missing {
            actual: relative_to(&actual, &folder),
        },
        ReadError::NotText(reason) => SnippetError::NotText(reason),
    })?;
    if !file.tags.problems.is_empty() {
        return Err(SnippetError::Tags(file));
    }
    let Some(extracted) = file.extract(address.region.as_deref()) else {
        return Err(SnippetError::RegionMissing(file));
    };
    Ok(Snippet {
        address: snippet.written.clone(),
        path: file.path.clone(),
        file: file.id,
        info: snippet.info(&file),
        code: extracted.code,
        lines: extracted.lines,
    })
}

/// `path` relative to `folder`, when it's inside it.
fn relative_to(path: &RelPath, folder: &RelPath) -> Option<String> {
    if !path.starts_with(folder) {
        return None;
    }
    let rest: Vec<&str> = path.segments().skip(folder.segments().count()).collect();
    Some(rest.join("/"))
}

/// The file-level issues of a `@snippet` (SPEC §8.2), reported at its
/// primary: none when it gives a snippet. Problems with the code file's tags
/// get the tag's line as related information.
pub fn snippet_issues(
    snippet: &SnippetUse,
    model: &ContentModel,
    fs: &dyn FileSystem,
    code: &CodeFiles,
    file: FileId,
) -> Vec<Issue> {
    let (Some(primary), Some(address)) = (snippet.primary, &snippet.address) else {
        // A missing primary is the parser's to report.
        return Vec::new();
    };
    let at = Location::new(file, primary);
    let address = match address {
        Ok(address) => address,
        Err(AddressError::NoSource) => {
            return vec![
                Issue::new(diagnostics::SNIPPET_ADDRESS, at)
                    .with_variant("no-source")
                    .with_arg("address", snippet.written.clone()),
            ];
        }
        Err(AddressError::Invalid(detail)) => {
            return vec![
                Issue::new(diagnostics::SNIPPET_ADDRESS, at)
                    .with_arg("address", snippet.written.clone())
                    .with_arg("detail", *detail),
            ];
        }
    };
    let Err(error) = resolve_snippet(snippet, address, model, fs, code) else {
        return Vec::new();
    };
    let named = address.file();
    match error {
        SnippetError::UnknownSource => {
            let names: Vec<&str> = model.sources.iter().map(|s| s.name.as_str()).collect();
            let issue = Issue::new(diagnostics::SNIPPET_SOURCE_UNKNOWN, at)
                .with_arg("source", address.source.clone());
            let issue = if names.is_empty() {
                issue.with_variant("none")
            } else if let Some(s) = suggest(&address.source, names.iter().copied()) {
                issue.with_variant("suggestion").with_arg("suggestion", s)
            } else {
                issue.with_arg("sources", names.join(", "))
            };
            vec![issue]
        }
        SnippetError::Missing { actual } => {
            let issue = Issue::new(diagnostics::SNIPPET_FILE_MISSING, at)
                .with_arg("path", address.path.clone())
                .with_arg("source", address.source.clone());
            let remote = model
                .source(&address.source)
                .is_some_and(|s| s.git.is_some());
            vec![match actual {
                Some(actual) => issue.with_variant("case").with_arg("actual", actual),
                // A source in another repository is read through its copies
                // (SPEC §7.4), which are made by a command of their own.
                None if remote => issue.with_variant("no-copy"),
                None => issue,
            }]
        }
        SnippetError::NotIncluded => vec![
            Issue::new(diagnostics::SNIPPET_FILE_MISSING, at)
                .with_variant("not-included")
                .with_arg("path", address.path.clone())
                .with_arg("source", address.source.clone()),
        ],
        SnippetError::Link => vec![
            Issue::new(diagnostics::SNIPPET_FILE_MISSING, at)
                .with_variant("link")
                .with_arg("path", address.path.clone())
                .with_arg("source", address.source.clone()),
        ],
        SnippetError::NotText(reason) => vec![
            Issue::new(diagnostics::SNIPPET_FILE_NOT_TEXT, at)
                .with_arg("path", named)
                .with_arg("reason", reason),
        ],
        SnippetError::Tags(code_file) => code_file
            .tags
            .problems
            .iter()
            .map(|p| tag_issue(p, &code_file, &named, at))
            .collect(),
        SnippetError::RegionMissing(code_file) => {
            let region = address.region.clone().unwrap_or_default();
            let issue = Issue::new(diagnostics::SNIPPET_REGION_MISSING, at)
                .with_arg("path", named)
                .with_arg("region", region.clone());
            let names = code_file.region_names();
            let issue = if code_file.style.is_none() {
                issue.with_variant("no-comments")
            } else if names.is_empty() {
                issue.with_variant("none")
            } else {
                let listed = names.join(", ");
                match suggest(&region, names.iter().copied()) {
                    Some(s) => issue
                        .with_variant("suggestion")
                        .with_arg("suggestion", s)
                        .with_arg("regions", listed),
                    None => issue.with_arg("regions", listed),
                }
            };
            vec![issue]
        }
    }
}

fn tag_issue(problem: &TagProblem, file: &CodeFile, named: &str, at: Location) -> Issue {
    let issue = Issue::new(diagnostics::SNIPPET_TAGS, at).with_arg("path", named);
    let here = Location::new(file.id, problem.span());
    let line = |i: &usize| (i + 1).to_string();
    match problem {
        TagProblem::Unclosed { tag, line: l, .. } => issue
            .with_arg("tag", tag.clone())
            .with_arg("line", line(l))
            .with_related(here, "opened here"),
        TagProblem::Unmatched { tag, line: l, .. } => issue
            .with_variant("unmatched")
            .with_arg("tag", tag.clone())
            .with_arg("line", line(l))
            .with_related(here, "nothing to close"),
        TagProblem::Duplicate {
            name,
            first,
            line: l,
            first_span,
            ..
        } => issue
            .with_variant("duplicate")
            .with_arg("region", name.clone())
            .with_arg("first", line(first))
            .with_arg("line", line(l))
            .with_related(Location::new(file.id, *first_span), "first started here")
            .with_related(here, "started again here"),
        TagProblem::Name { line: l, .. } => issue
            .with_variant("name")
            .with_arg("line", line(l))
            .with_related(here, "this tag"),
        TagProblem::Reserved { tag, line: l, .. } => issue
            .with_variant("reserved")
            .with_arg("tag", tag.clone())
            .with_arg("line", line(l))
            .with_related(here, "this tag"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        let a = parse_address("code:examples/quill/ascribe.toml#dimensions").expect("valid");
        assert_eq!(a.source, "code");
        assert_eq!(a.path, "examples/quill/ascribe.toml");
        assert_eq!(a.region.as_deref(), Some("dimensions"));
        assert_eq!(a.to_string(), "code:examples/quill/ascribe.toml#dimensions");
        assert_eq!(a.file(), "code:examples/quill/ascribe.toml");

        let a = parse_address("my-code:a%20b.rs").expect("valid");
        assert_eq!(a.path, "a b.rs");
        assert_eq!(a.region, None);

        assert_eq!(parse_address("../x.rs"), Err(AddressError::NoSource));
        assert_eq!(parse_address("x.rs#a"), Err(AddressError::NoSource));
        for bad in [
            "Code:x.rs",
            "code:",
            "code:#a",
            "code:/x.rs",
            "code:a//b.rs",
            "code:../x.rs",
            "code:a/./b.rs",
            "code:a\\b.rs",
            "code:x.rs#",
            "code:x.rs#a b",
            "code:x.rs#a/b",
        ] {
            assert!(
                matches!(parse_address(bad), Err(AddressError::Invalid(_))),
                "{bad}"
            );
        }
    }
}
