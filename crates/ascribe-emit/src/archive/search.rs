//! An archive's search: the index `ascribe versions export` writes beside
//! the pages, and the script that searches it from the box in each page's
//! sidebar (decision #502j, `project-docs/decisions/502j-archive.md`).
//!
//! A page opened from a disk can't fetch another file, so the index is a
//! script, `search-index.js`, that sets [`INDEX_GLOBAL`] to the index as a
//! JSON object; the search script adds it to the page with a `<script>`
//! element the first time the box is used. The index is each page's title,
//! and its text by heading, each heading with its id, from the resolved
//! page and not from its HTML: paragraphs, list items, table cells, code
//! blocks, and directives' titles, whitespace collapsed. Raw HTML is left
//! out.

use ascribe_core::RelPath;
use ascribe_resolve::{ResolvedBlock, ResolvedKind, ResolvedPage};
use ascribe_syntax::{BlockKind, DirectiveLine, PrimaryValue};
use serde::Serialize;

use crate::assets::encode_path;
use crate::labels::plain_text;

/// The search script, at the top of the folder.
pub const SEARCH_SCRIPT: &str = "search.js";
/// The index the script searches, at the top of the folder.
pub const SEARCH_INDEX: &str = "search-index.js";
/// The global `search-index.js` sets to the index.
pub const INDEX_GLOBAL: &str = "ascribeArchiveIndex";
/// The size of an index past which the command says it's large, in bytes:
/// a reader's first search waits for it to load (decision 77).
pub const LARGE_SEARCH_INDEX: usize = 8_000_000;

/// The search script: `packages/archive/src/search.ts`, bundled by
/// `pnpm --filter @ascribed/archive embed`.
pub(super) const SCRIPT: &str = include_str!("search.js");

/// The version of the index's shape, which the script reads.
const INDEX_VERSION: u32 = 1;

/// The index: every page in reading order.
#[derive(Serialize)]
struct Index<'a> {
    version: u32,
    pages: &'a [IndexPage],
}

/// A page of the index.
#[derive(Serialize)]
pub(super) struct IndexPage {
    /// Its file's address from the top of the folder.
    href: String,
    /// Its title, as text.
    title: String,
    /// Its text by heading, the text before its first heading first.
    sections: Vec<Section>,
}

/// A heading and the text under it, up to the next heading.
#[derive(Serialize, Default)]
struct Section {
    /// The heading's id on the page: empty for the text before the first
    /// heading, and for a heading with none.
    id: String,
    /// The heading, as text: empty for the text before the first heading.
    heading: String,
    /// The text under it, whitespace collapsed.
    text: String,
}

/// A page as the index holds it, written at `output`.
pub(super) fn index_page(page: &ResolvedPage, output: &RelPath, title: String) -> IndexPage {
    let mut collector = Collector {
        sections: vec![Section::default()],
    };
    collector.blocks(&page.blocks);
    let sections = collector
        .sections
        .into_iter()
        .map(|mut s| {
            s.text = collapse(&s.text);
            s
        })
        .filter(|s| !s.heading.is_empty() || !s.text.is_empty())
        .collect();
    IndexPage {
        href: encode_path(output.as_str()),
        title,
        sections,
    }
}

/// `search-index.js`: the script that sets [`INDEX_GLOBAL`] to the index.
pub(super) fn index_script(pages: &[IndexPage]) -> String {
    let index = Index {
        version: INDEX_VERSION,
        pages,
    };
    // Serializing a struct of strings and numbers can't fail.
    let json = serde_json::to_string(&index).unwrap_or_else(|_| "null".to_owned());
    format!(
        "/* The search index of an archive, written by `ascribe versions export`. */\nglobalThis.{INDEX_GLOBAL} = {json};\n"
    )
}

/// Gathers a page's text by heading.
struct Collector {
    sections: Vec<Section>,
}

impl Collector {
    fn blocks(&mut self, blocks: &[ResolvedBlock]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn push(&mut self, text: &str) {
        if let Some(section) = self.sections.last_mut() {
            if !section.text.is_empty() {
                section.text.push(' ');
            }
            section.text.push_str(text);
        }
    }

    fn directive(&mut self, line: &DirectiveLine) {
        if let Some(title) = &line.title {
            self.push(&plain_text(&title.inlines));
        }
        if let Some(PrimaryValue::Text(text)) = &line.primary {
            self.push(&plain_text(&text.inlines));
        }
    }

    fn block(&mut self, block: &ResolvedBlock) {
        match &block.kind {
            ResolvedKind::Leaf(leaf) => match &leaf.kind {
                BlockKind::Heading(heading) => {
                    let (id, text) = match &block.heading {
                        Some(ids) => (ids.page_id.clone(), ids.text.clone()),
                        None => (String::new(), plain_text(&heading.inlines)),
                    };
                    self.sections.push(Section {
                        id,
                        heading: collapse(&text),
                        text: String::new(),
                    });
                }
                BlockKind::Paragraph(paragraph) => self.push(&plain_text(&paragraph.inlines)),
                BlockKind::CodeBlock(code) => self.push(&code.literal),
                BlockKind::Table(table) => {
                    for cell in table.rows.iter().flat_map(|r| &r.cells) {
                        self.push(&plain_text(&cell.inlines));
                    }
                }
                BlockKind::Directive(line) => self.directive(line),
                _ => {}
            },
            ResolvedKind::BlockQuote { children } => self.blocks(children),
            ResolvedKind::List { items, .. } => {
                for item in items {
                    self.blocks(&item.children);
                }
            }
            ResolvedKind::Container {
                opener, children, ..
            } => {
                self.directive(opener);
                self.blocks(children);
            }
            ResolvedKind::Group { arms, .. } => {
                for arm in arms {
                    self.directive(&arm.opener);
                    self.blocks(&arm.children);
                }
            }
        }
    }
}

/// Runs of whitespace as one space, and none at either end.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_reads_the_global_the_index_sets() {
        assert!(SCRIPT.contains(&format!("globalThis.{INDEX_GLOBAL}")));
    }

    #[test]
    fn the_index_is_a_script_that_sets_its_global() {
        let script = index_script(&[IndexPage {
            href: "guide/install.html".into(),
            title: "Install".into(),
            sections: vec![Section {
                id: "connect".into(),
                heading: "Connect".into(),
                text: "Run quill --token".into(),
            }],
        }]);
        assert!(script.ends_with(
            "globalThis.ascribeArchiveIndex = {\"version\":1,\"pages\":[{\"href\":\"guide/install.html\",\"title\":\"Install\",\"sections\":[{\"id\":\"connect\",\"heading\":\"Connect\",\"text\":\"Run quill --token\"}]}]};\n"
        ));
    }
}
