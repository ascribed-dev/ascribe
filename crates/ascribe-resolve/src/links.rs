//! Writing a link: the destination a page writes for another file, and the
//! headings a link to a file can name. The editor's completion and actions
//! and the `ascribe` commands that answer questions about links all write
//! links with these, so they write them the same way.

use std::collections::HashSet;

use ascribe_core::RelPath;

use crate::index::{FileIndex, Heading};
use crate::project::Project;

impl Project {
    /// The headings a link to `file` can name, in document order, each with
    /// the content path of the file it's written in: a page's own and those
    /// of the fragments it includes (SPEC §4.2); a fragment's own. Each id
    /// once, the first heading with it winning, and none without an id.
    pub fn link_headings<'a>(&'a self, file: &'a FileIndex) -> Vec<(RelPath, &'a Heading)> {
        let all: Vec<(RelPath, &Heading)> = match self.expansion(&file.path) {
            Some(page) => page.headings(self),
            None => file
                .headings
                .iter()
                .map(|h| (file.path.clone(), h))
                .collect(),
        };
        let mut seen = HashSet::new();
        all.into_iter()
            .filter(|(_, h)| !h.source_id.is_empty() && seen.insert(h.source_id.clone()))
            .collect()
    }
}

/// The headings with an id, each id once, the first with it winning.
pub fn named_headings<'a>(headings: impl IntoIterator<Item = &'a Heading>) -> Vec<&'a Heading> {
    let mut seen = HashSet::new();
    headings
        .into_iter()
        .filter(|h| !h.source_id.is_empty() && seen.insert(h.source_id.as_str()))
        .collect()
}

/// Percent-encodes what would end a link destination or an include path early
/// (whitespace, parentheses, angle brackets) or would be read as an escape.
pub fn encode_destination(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            c => out.push(c),
        }
    }
    out
}

/// The path from the directory of `from` to `target`, as it is written in a
/// link or an include: `keys.md`, `../keys.md`, `sub/page.md`.
pub fn link_path(target: &RelPath, from: &RelPath) -> String {
    let dir = from.parent().unwrap_or_default();
    match target.relative_from(&dir) {
        Some(text) => text.strip_prefix("./").map_or(text.clone(), str::to_owned),
        None => format!("/{target}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destinations_are_encoded() {
        assert_eq!(
            encode_destination("My Setup (1).md"),
            "My%20Setup%20%281%29.md"
        );
        assert_eq!(encode_destination("a/b.md"), "a/b.md");
        assert_eq!(encode_destination("hash#name.md"), "hash%23name.md");
    }

    #[test]
    fn link_paths_have_no_dot_prefix() {
        let p = |s: &str| RelPath::parse(s).expect("a path");
        assert_eq!(link_path(&p("a/b.md"), &p("a/c.md")), "b.md");
        assert_eq!(link_path(&p("keys.md"), &p("a/c.md")), "../keys.md");
        assert_eq!(link_path(&p("a/b/x.md"), &p("a/c.md")), "b/x.md");
        assert_eq!(link_path(&p("keys.md"), &p("index.md")), "keys.md");
    }
}
