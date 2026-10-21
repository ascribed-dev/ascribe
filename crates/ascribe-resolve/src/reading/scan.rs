//! The source as written, searched by the same rules as a reading: what
//! `ascribe search` looks in when nothing a build publishes matches, to say
//! whether the words are written at all.
//!
//! Each page's, fragment's, and the content model's text is read in blocks
//! separated by blank lines, its white space collapsed, and a query's words
//! must all be in one of them. Markup is read as written, so `{key}` is a
//! phrase's key, not its value. Memos are left out, as everything leaves
//! them out ([decision #536c]).
//!
//! [decision #536c]: https://github.com/ascribed-dev/ascribe/blob/main/project-docs/decisions/536c-nothing-reads-a-memo.md

use std::collections::HashSet;

use ascribe_core::schema::Builtin;
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_syntax::{Block, BlockKind};

use super::{Query, line_col};
use crate::Project;
use crate::fs::MODEL_FILE;

/// A line of the source that writes what a query matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    /// The file, from the project root.
    pub file: RelPath,
    /// The line where the match starts, from 1.
    pub line: u32,
}

/// The lines of `project`'s pages and fragments, and of its content model,
/// whose text is `model`, where `query` matches as written: one for each
/// block of text it matches in, in path order and then in order within a
/// file. With `only`, of the pages and fragments it names, by content path,
/// and the content model.
pub fn scan(
    project: &Project,
    model: &str,
    query: &Query,
    only: Option<&HashSet<RelPath>>,
) -> Vec<Written> {
    let mut files: Vec<(RelPath, &str, Vec<Span>)> = project
        .files()
        .filter(|file| only.is_none_or(|only| only.contains(&file.path)))
        .map(|file| {
            let mut memos = Vec::new();
            memos_in(&file.document.blocks, &mut memos);
            (
                project.layout().project_path(&file.path),
                &*file.source,
                memos,
            )
        })
        .collect();
    if let Ok(path) = RelPath::parse(MODEL_FILE) {
        files.push((path, model, Vec::new()));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::new();
    for (path, text, memos) in files {
        let index = LineIndex::new(text);
        for chunk in chunks(text, &memos) {
            if let Some(found) = query.find_in(&chunk.text) {
                let at = chunk.offsets.get(chunk.text[..found.start].chars().count());
                let (line, _) = line_col(&index, at.copied().unwrap_or_default());
                out.push(Written {
                    file: path.clone(),
                    line,
                });
            }
        }
    }
    out
}

/// A block of a file's text with its white space collapsed, and where each
/// of its characters is written.
struct Chunk {
    text: String,
    offsets: Vec<usize>,
}

/// `text`'s blocks, split at blank lines and at memos, which are left out.
fn chunks(text: &str, memos: &[Span]) -> Vec<Chunk> {
    let mut out = Vec::new();
    let mut chunk = Chunk {
        text: String::new(),
        offsets: Vec::new(),
    };
    let mut flush = |chunk: &mut Chunk| {
        if chunk.text.ends_with(' ') {
            chunk.text.pop();
            chunk.offsets.pop();
        }
        if !chunk.text.is_empty() {
            out.push(std::mem::replace(
                chunk,
                Chunk {
                    text: String::new(),
                    offsets: Vec::new(),
                },
            ));
        }
    };
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if line.trim().is_empty() {
            flush(&mut chunk);
            continue;
        }
        for (i, c) in line.char_indices() {
            let at = start + i;
            if memos.iter().any(|m| m.start() <= at && at < m.end()) {
                flush(&mut chunk);
                continue;
            }
            if c.is_whitespace() {
                if !chunk.text.is_empty() && !chunk.text.ends_with(' ') {
                    chunk.text.push(' ');
                    chunk.offsets.push(at);
                }
            } else {
                chunk.text.push(c);
                chunk.offsets.push(at);
            }
        }
    }
    flush(&mut chunk);
    out
}

/// The spans of the memos in `blocks`, at any depth, in either form.
fn memos_in(blocks: &[Block], out: &mut Vec<Span>) {
    let memo = Builtin::Memo.name();
    for block in blocks {
        match &block.kind {
            BlockKind::Directive(line) if line.name == memo => out.push(block.span),
            BlockKind::Container(c) if c.opener.name == memo => out.push(block.span),
            BlockKind::Container(c) => memos_in(&c.children, out),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    memos_in(&arm.children, out);
                }
            }
            BlockKind::BlockQuote(q) => memos_in(&q.children, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    memos_in(&item.children, out);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn texts(text: &str, memos: &[Span]) -> Vec<String> {
        chunks(text, memos).into_iter().map(|c| c.text).collect()
    }

    #[test]
    fn blocks_are_split_at_blank_lines_with_white_space_collapsed() {
        assert_eq!(
            texts("One  two\nthree\n\n  \nfour\n", &[]),
            ["One two three", "four"]
        );
    }

    #[test]
    fn a_memo_is_left_out_and_splits_its_block() {
        let text = "Before\n@memo: secret\nafter\n";
        let memo = Span::new(7, 20);
        assert_eq!(texts(text, &[memo]), ["Before", "after"]);
    }

    #[test]
    fn each_character_is_at_its_offset() {
        let text = "a\n  bé c";
        let chunk = chunks(text, &[]).remove(0);
        assert_eq!(chunk.text, "a bé c");
        let at: Vec<&str> = chunk
            .offsets
            .iter()
            .map(|&o| &text[o..o + text[o..].chars().next().unwrap().len_utf8()])
            .collect();
        assert_eq!(at, ["a", "\n", "b", "é", " ", "c"]);
    }
}

/// What the source says about words nothing a build searched publishes.
#[derive(Debug, PartialEq, Eq)]
pub enum Nothing {
    /// No page, fragment, or `ascribe.toml` has them as written.
    Unwritten,
    /// They're written, but no build searched publishes them there, or a
    /// phrase's value hides them.
    Unpublished {
        /// The first places they're written, at most three: each file, from
        /// the project root, and line.
        written: Vec<(String, u32)>,
        /// How many places they're written.
        total: usize,
    },
    /// Builds that weren't searched publish them, in the content model's
    /// order.
    OtherBuilds(Vec<String>),
}

impl Nothing {
    /// What it says, on one line: the last line of `ascribe search`'s text
    /// when nothing matched, and what the editor's search shows.
    pub fn said(&self) -> String {
        match self {
            Nothing::Unwritten => "The words aren't in any page.".to_owned(),
            Nothing::Unpublished { written, total } => {
                let first = written
                    .first()
                    .map_or_else(String::new, |(file, line)| format!("`{file}:{line}`"));
                let mut text =
                    format!("No build searched publishes them. They're written at {first}");
                let more = total.saturating_sub(1);
                if more > 0 {
                    let places = if more == 1 { "place" } else { "places" };
                    text.push_str(&format!(" and {more} more {places}"));
                }
                text.push('.');
                text
            }
            Nothing::OtherBuilds(builds) => {
                let builds: Vec<String> = builds.iter().map(|b| format!("`{b}`")).collect();
                format!("Builds not searched publish them: {}.", builds.join(", "))
            }
        }
    }
}
