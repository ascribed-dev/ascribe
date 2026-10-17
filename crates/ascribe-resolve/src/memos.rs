//! Memo files (SPEC §7.7): a memo longer than a line or two, kept in a file
//! of its own, never published, and placed on the page, section, or block
//! it's about each time it's read.
//!
//! A [`MemoStore`] gives a project's memo files, each parsed on its own
//! ([`MemoFile::parse`]): frontmatter with `about` and `quote`, then a body
//! kept as text, with nothing in it resolved. [`Committed`] is the store of
//! the committed folder, `memos/` beside `ascribe.toml`, and [`Personal`] the
//! store of the memos kept on one machine, `.ascribe/memos/`, both read
//! through the project's [`FileSystem`] in the same format. [`place`] works out where a memo is against the
//! source index, every time: nothing is cached, and nothing is written back.

use ascribe_core::schema::{Builtin, builtin_schemas};
use ascribe_core::{RelPath, Span};
use ascribe_syntax::{Block, BlockKind, ParseOptions, frontmatter_lines, parse, raw_text};

use crate::fs::FileSystem;
use crate::index::walk::walk_blocks;
use crate::layout::Layout;
use crate::named::Named;
use crate::project::Project;
use crate::uses::Usable;

/// The committed memos' folder, beside `ascribe.toml`.
pub const MEMOS_FOLDER: &str = "memos";

/// The personal memos' folder, in the project's own folder, which the
/// project doesn't commit.
pub const PERSONAL_MEMOS_FOLDER: &str = ".ascribe/memos";

/// The frontmatter keys a memo file has.
pub const MEMO_KEYS: &[&str] = &["about", "quote"];

/// One memo file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoFile {
    /// Its path from the project root.
    pub path: RelPath,
    /// Its text; empty when it couldn't be read.
    pub text: String,
    /// `about`, when it's there and is text.
    pub about: Option<Key>,
    /// `quote`, when it's there and is text.
    pub quote: Option<Key>,
    /// What `about` names, when it's written as a target is.
    pub named: Option<Named>,
    /// The body: everything after the frontmatter, without the blank lines
    /// before it.
    pub body: Span,
    /// What's wrong with it, in source order.
    pub problems: Vec<MemoProblem>,
}

/// A frontmatter key's value, and where it's written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    /// The value, as YAML reads it.
    pub value: String,
    /// The key's line, from the key to the end of the value.
    pub line: Span,
}

/// Something wrong with a memo file, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoProblem {
    /// What it is.
    pub kind: Invalid,
    /// Where: the line of the key it's about, or the file's first line.
    pub span: Span,
}

/// The ways a memo file can be wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invalid {
    /// It can't be read, or isn't UTF-8: the reason.
    Unreadable(String),
    /// It doesn't start with frontmatter.
    NoFrontmatter,
    /// Its frontmatter isn't YAML, or isn't a mapping: the parser's reason.
    Yaml(String),
    /// It has no `about`.
    NoAbout,
    /// A key a memo file doesn't have.
    UnknownKey(String),
    /// `about` or `quote` isn't text, or is empty: the key.
    NotText(&'static str),
    /// `about` isn't written as a target is: why.
    BadAbout(String),
    /// `quote` with an `about` that names a content model entry, which has
    /// no text to quote.
    QuoteWithoutText,
}

impl MemoFile {
    /// Parses a memo file at `path`, from the project root.
    pub fn parse(path: RelPath, text: String) -> MemoFile {
        let mut memo = MemoFile {
            path,
            about: None,
            quote: None,
            named: None,
            body: Span::new(text.len(), text.len()),
            problems: Vec::new(),
            text: String::new(),
        };
        let document = parse(&text, &ParseOptions::new(builtin_schemas()));
        let first_line = Span::new(0, text.find('\n').unwrap_or(text.len()));
        let Some(frontmatter) = document.frontmatter else {
            memo.body = body(&text, 0);
            memo.problem(Invalid::NoFrontmatter, first_line);
            memo.text = text;
            return memo;
        };
        memo.body = body(&text, frontmatter.span.end());
        let yaml = text.get(frontmatter.content.range()).unwrap_or("");
        let mapping = match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(yaml) {
            Ok(serde_yaml_ng::Value::Mapping(map)) => map,
            Ok(serde_yaml_ng::Value::Null) => Default::default(),
            Ok(_) => {
                memo.problem(
                    Invalid::Yaml("it isn't a mapping of keys to values".to_owned()),
                    first_line,
                );
                memo.text = text;
                return memo;
            }
            Err(err) => {
                memo.problem(Invalid::Yaml(err.to_string()), first_line);
                memo.text = text;
                return memo;
            }
        };
        // Each top-level key's line, by key.
        let lines: Vec<(&str, Span)> = frontmatter_lines(yaml)
            .into_iter()
            .filter(|l| l.indent == 0 && l.item.is_none())
            .filter_map(|l| {
                let start = frontmatter.content.start() + l.start;
                let end = frontmatter.content.start() + l.value_start + l.value.len();
                Some((l.key?, Span::new(start, end.max(start))))
            })
            .collect();
        let line_of = |key: &str| {
            lines
                .iter()
                .find(|(k, _)| *k == key)
                .map_or(first_line, |(_, span)| *span)
        };
        for key in mapping.keys() {
            let name = key.as_str().map_or_else(
                || {
                    serde_yaml_ng::to_string(key)
                        .unwrap_or_default()
                        .trim()
                        .to_owned()
                },
                str::to_owned,
            );
            if !MEMO_KEYS.contains(&name.as_str()) {
                let at = line_of(&name);
                memo.problem(Invalid::UnknownKey(name), at);
            }
        }
        let text_key = |memo: &mut MemoFile, key: &'static str| -> Option<Key> {
            let value = mapping.get(key)?;
            let line = line_of(key);
            match value.as_str().map(str::trim) {
                Some(value) if !value.is_empty() => Some(Key {
                    value: value.to_owned(),
                    line,
                }),
                _ => {
                    memo.problem(Invalid::NotText(key), line);
                    None
                }
            }
        };
        memo.about = text_key(&mut memo, "about");
        memo.quote = text_key(&mut memo, "quote");
        match &memo.about {
            None if !mapping.contains_key("about") => {
                let opening = Span::new(0, text.find('\n').unwrap_or(text.len()));
                memo.problem(Invalid::NoAbout, opening);
            }
            None => {}
            Some(about) => match Named::parse(&about.value) {
                Ok(named) => memo.named = Some(named),
                Err(bad) => {
                    let at = about.line;
                    memo.problem(Invalid::BadAbout(bad.reason), at);
                }
            },
        }
        if let (Some(Named::Entry { .. }), Some(quote)) = (&memo.named, &memo.quote) {
            let at = quote.line;
            memo.problem(Invalid::QuoteWithoutText, at);
        }
        memo.problems.sort_by_key(|p| p.span);
        memo.text = text;
        memo
    }

    /// A memo file that couldn't be read.
    pub fn unreadable(path: RelPath, reason: String) -> MemoFile {
        MemoFile {
            path,
            text: String::new(),
            about: None,
            quote: None,
            named: None,
            body: Span::new(0, 0),
            problems: vec![MemoProblem {
                kind: Invalid::Unreadable(reason),
                span: Span::new(0, 0),
            }],
        }
    }

    /// Whether it's a personal memo, kept in `.ascribe/memos/` on one
    /// machine, rather than one of the project's `memos/`.
    pub fn personal(&self) -> bool {
        RelPath::parse(PERSONAL_MEMOS_FOLDER).is_ok_and(|folder| self.path.starts_with(&folder))
    }

    /// The body's text.
    pub fn body_text(&self) -> &str {
        self.text.get(self.body.range()).unwrap_or("")
    }

    /// The body's first line that isn't blank, trimmed: what a list shows of
    /// the memo.
    pub fn first_line(&self) -> &str {
        self.body_text()
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("")
    }

    fn problem(&mut self, kind: Invalid, span: Span) {
        self.problems.push(MemoProblem { kind, span });
    }
}

/// The body of `text` after `from`: the rest, without the blank lines
/// before it or the whitespace after it.
fn body(text: &str, from: usize) -> Span {
    let rest = text.get(from..).unwrap_or("");
    let start = from + (rest.len() - rest.trim_start().len());
    let end = from + rest.trim_end().len();
    Span::new(start, end.max(start))
}

/// Where memo files come from.
pub trait MemoStore {
    /// Every memo file, in path order.
    fn memos(&self) -> Vec<MemoFile>;
}

/// The committed folder, `memos/` beside `ascribe.toml`: every `.md` file in
/// it and its subfolders, skipping any whose name, or a folder's on the way
/// to it, starts with `.`.
pub struct Committed<'a> {
    fs: &'a dyn FileSystem,
    layout: &'a Layout,
}

impl<'a> Committed<'a> {
    /// The committed memos of the project `fs` and `layout` describe.
    pub fn new(fs: &'a dyn FileSystem, layout: &'a Layout) -> Committed<'a> {
        Committed { fs, layout }
    }
}

impl MemoStore for Committed<'_> {
    /// None when the folder is inside the content root: its files are
    /// pages ([`folder_in_content`]).
    fn memos(&self) -> Vec<MemoFile> {
        if folder_in_content(self.layout).is_some() {
            return Vec::new();
        }
        read_folder(self.fs, MEMOS_FOLDER)
    }
}

/// The personal folder, `.ascribe/memos/` beside `ascribe.toml`: the memos
/// kept on one machine, in the committed folder's format, and read the same
/// way. `ascribe check` doesn't read them, so what it reports is the same on
/// every machine.
pub struct Personal<'a> {
    fs: &'a dyn FileSystem,
}

impl<'a> Personal<'a> {
    /// The personal memos of the project `fs` reads.
    pub fn new(fs: &'a dyn FileSystem) -> Personal<'a> {
        Personal { fs }
    }
}

impl MemoStore for Personal<'_> {
    fn memos(&self) -> Vec<MemoFile> {
        read_folder(self.fs, PERSONAL_MEMOS_FOLDER)
    }
}

/// Every `.md` file in `folder`, from the project root, and its subfolders,
/// in path order, skipping any whose name, or a folder's on the way to it
/// from `folder`, starts with `.`.
fn read_folder(fs: &dyn FileSystem, folder: &str) -> Vec<MemoFile> {
    let Ok(folder) = RelPath::parse(folder) else {
        return Vec::new();
    };
    let mut paths: Vec<RelPath> = fs
        .files_in(&folder)
        .into_iter()
        .filter(|p| p.extension() == Some("md"))
        .filter(|p| {
            !p.segments()
                .skip(folder.segments().count())
                .any(|s| s.starts_with('.'))
        })
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| match fs.read_file(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => MemoFile::parse(path, text),
                Err(_) => MemoFile::unreadable(path, "it isn't UTF-8".to_owned()),
            },
            Err(err) => {
                let reason = err.to_string();
                MemoFile::unreadable(path, reason)
            }
        })
        .collect()
}

/// Whether `path`, from the project root, is a memo file either store
/// reads: a `.md` file in `memos/`, when that folder isn't in the content
/// root, or in `.ascribe/memos/`, with no name on the way from the folder
/// starting with `.`.
pub fn is_memo_path(path: &RelPath, layout: &Layout) -> bool {
    if path.extension() != Some("md") {
        return false;
    }
    let committed = folder_in_content(layout).is_none().then_some(MEMOS_FOLDER);
    [committed, Some(PERSONAL_MEMOS_FOLDER)]
        .into_iter()
        .flatten()
        .filter_map(|folder| RelPath::parse(folder).ok())
        .any(|folder| {
            path.starts_with(&folder)
                && *path != folder
                && !path
                    .segments()
                    .skip(folder.segments().count())
                    .any(|s| s.starts_with('.'))
        })
}

/// The committed memos' folder as a content path, when it's inside the
/// content root, as it is when the content root is the project root: its
/// files are read as pages then, and none as a memo.
pub fn folder_in_content(layout: &Layout) -> Option<RelPath> {
    let folder = RelPath::parse(MEMOS_FOLDER).ok()?;
    let content = folder.relative_to(&layout.content_root)?;
    content.is_inside().then_some(content)
}

/// Where a memo is, worked out each time it's read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    /// What its `about` names.
    pub named: Named,
    /// What that is in the project; `None` when it names nothing.
    pub target: Option<Usable>,
    /// Which of the three it is.
    pub state: MemoState,
    /// With a `quote` that's found, the block that holds it, in the file
    /// the target is written in.
    pub block: Option<Span>,
}

/// A memo's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoState {
    /// On its target, or on the block its quote is in.
    Placed,
    /// Its quote isn't in its target: it stays on its page or section.
    Outdated,
    /// Its `about` names nothing in the project.
    Detached,
}

impl MemoState {
    /// Its name, as the commands' JSON writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            MemoState::Placed => "placed",
            MemoState::Outdated => "outdated",
            MemoState::Detached => "detached",
        }
    }
}

/// Places `memo` against `project`'s source index: `None` when its `about`
/// isn't a target, or it has a quote and its target has no text.
///
/// With no `quote`, the memo is on its target. With one, the blocks of the
/// target are searched in source order, each block's source as written in
/// the file it names, with whitespace runs read as one space and case kept;
/// the first that holds it is the memo's block. A quote across two blocks
/// isn't found.
pub fn place(memo: &MemoFile, project: &Project) -> Option<Placement> {
    let named = memo.named.clone()?;
    if matches!(named, Named::Entry { .. }) && memo.quote.is_some() {
        return None;
    }
    let Some(target) = named.find(project) else {
        return Some(Placement {
            named,
            target: None,
            state: MemoState::Detached,
            block: None,
        });
    };
    let Some(quote) = &memo.quote else {
        return Some(Placement {
            named,
            target: Some(target),
            state: MemoState::Placed,
            block: None,
        });
    };
    let block = quoted_block(project, &target, &quote.value);
    Some(Placement {
        named,
        state: if block.is_some() {
            MemoState::Placed
        } else {
            MemoState::Outdated
        },
        target: Some(target),
        block,
    })
}

/// The first block of `target` whose source holds `quote`.
fn quoted_block(project: &Project, target: &Usable, quote: &str) -> Option<Span> {
    let (path, within) = match target {
        Usable::File(path) => (path, None),
        Usable::Heading { file, id } => (file, Some(project.heading(file, id)?.section)),
        _ => return None,
    };
    let file = project.file(path)?;
    let quote = spaced(quote);
    let mut found = None;
    walk_blocks(&file.document.blocks, &mut |block| {
        if found.is_some() || !is_leaf(block) {
            return;
        }
        if within.is_some_and(|s| block.span.start() < s.start() || block.span.end() > s.end()) {
            return;
        }
        if spaced(&raw_text(&file.source, block.span)).contains(&quote) {
            found = Some(block.span);
        }
    });
    found
}

/// Whether a block holds text of its own, rather than blocks: not a block
/// quote, a list, a container, or a group, and not a memo, which is no
/// part of what a page says.
fn is_leaf(block: &Block) -> bool {
    match &block.kind {
        BlockKind::BlockQuote(_)
        | BlockKind::List(_)
        | BlockKind::Container(_)
        | BlockKind::Group(_) => false,
        BlockKind::Directive(line) => line.name != Builtin::Memo.name(),
        _ => true,
    }
}

/// `text` with each run of whitespace read as one space, and none at
/// either end.
fn spaced(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn memo(text: &str) -> MemoFile {
        MemoFile::parse(RelPath::parse("memos/a.md").unwrap(), text.to_owned())
    }

    fn kinds(memo: &MemoFile) -> Vec<Invalid> {
        memo.problems.iter().map(|p| p.kind.clone()).collect()
    }

    #[test]
    fn a_memo_has_about_quote_and_a_body() {
        let m = memo("---\nabout: a.md#b\nquote: Some  text\n---\n\nThe reason.\nMore.\n");
        assert!(m.problems.is_empty(), "{:?}", m.problems);
        assert_eq!(m.about.as_ref().unwrap().value, "a.md#b");
        assert_eq!(
            &m.text[m.about.as_ref().unwrap().line.range()],
            "about: a.md#b"
        );
        assert_eq!(
            &m.text[m.quote.as_ref().unwrap().line.range()],
            "quote: Some  text"
        );
        assert_eq!(m.body_text(), "The reason.\nMore.");
        assert_eq!(m.first_line(), "The reason.");
        assert_eq!(
            m.named,
            Some(Named::Path {
                path: RelPath::parse("a.md").unwrap(),
                id: Some("b".to_owned())
            })
        );
    }

    #[test]
    fn what_makes_a_memo_file_invalid() {
        assert_eq!(kinds(&memo("No frontmatter.\n")), [Invalid::NoFrontmatter]);
        assert_eq!(
            kinds(&memo("---\nquote: x\n---\nText.\n")),
            [Invalid::NoAbout]
        );
        assert_eq!(
            kinds(&memo("---\nabout: a.md\nauthor: me\n---\n")),
            [Invalid::UnknownKey("author".to_owned())]
        );
        assert_eq!(
            kinds(&memo("---\nabout: [a.md]\n---\n")),
            [Invalid::NotText("about")]
        );
        assert_eq!(
            kinds(&memo("---\nabout: a.md\nquote: \"\"\n---\n")),
            [Invalid::NotText("quote")]
        );
        assert!(matches!(
            kinds(&memo("---\nabout: \"phrase:\"\n---\n"))[..],
            [Invalid::BadAbout(_)]
        ));
        assert_eq!(
            kinds(&memo("---\nabout: phrase:product\nquote: Quill\n---\n")),
            [Invalid::QuoteWithoutText]
        );
        assert!(matches!(
            kinds(&memo("---\nabout: [\n---\n"))[..],
            [Invalid::Yaml(_)]
        ));
    }

    #[test]
    fn whitespace_runs_read_as_one_space() {
        assert_eq!(spaced("  a \n\t b  "), "a b");
    }
}
